/* copy the contents of objects into a final executable:
 * sections and relocations
 *
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::manifest::{ self, Manifest, FileIdentifier };
use super::config::{ Config, SECTIONS };

use object::{ Object, SymbolSection };
use object::{ ObjectSection, SectionIndex, write::SectionId, SectionKind };
use object::{ ObjectSymbol, SymbolIndex, write::SymbolId, SymbolKind, SymbolFlags };
use object::read::RelocationTarget;

use wildmatch::WildMatch;
use std::collections::HashMap;

/* describe a section within an object within the manifest */
#[derive(PartialEq, Eq, Hash)]
pub struct ManifestSection
{
    pub identifier: FileIdentifier,
    pub index: SectionIndex
}

/* describe a symbol within an object in the manifest */
#[derive(PartialEq, Eq, Hash)]
pub struct ManifestSymbol
{
    pub identifier: FileIdentifier,
    pub index: SymbolIndex
}

/* copy relocations in the manifest's object files into the executable */
pub fn relocations(elf: &mut object::write::Object, manifest: &Manifest, output_sections: &HashMap<ManifestSection, SectionId>, output_symbols: &HashMap<ManifestSymbol, SymbolId>)
{
    for (obj_name, mapping) in manifest.raw_objects()
    {
        /* spin through the sections in the manifest looking for relocations */
        let parsed = manifest::parse(mapping);
        for input_section in parsed.sections()
        {
            match input_section.kind()
            {
                SectionKind::Metadata => (), /* skip section types we're not interested in */
                _ =>
                {
                    /* find the section in the executable that corresponds to this section in the manifest */
                    let output_section = match output_sections.get(&ManifestSection
                    {
                        identifier: obj_name.clone(),
                        index: input_section.index()
                    })
                    {
                        Some(section) => *section,
                        None => continue /* if we skipped the output section, then ignore */
                    };

                    /* go through the relocations in each manifest object's sections */
                    for (offset, relocation) in input_section.relocations()
                    {
                        let output_symbol = match relocation.target()
                        {
                            /* transform a symbol index in the input section into
                               the symbol index in the corresponding output section */
                            RelocationTarget::Symbol(input_symbol_idx) =>
                            {
                                match output_symbols.get(&ManifestSymbol
                                {
                                    identifier: obj_name.clone(),
                                    index: input_symbol_idx
                                })
                                {
                                    Some(symbol) => *symbol,
                                    None => continue /* skip symbols we left out */
                                }
                            },

                            /* transform a section symbol in the input manifest object into
                               the section symbol in the output executable */
                            RelocationTarget::Section(input_section_idx) =>
                            {
                                let output_section_idx = match output_sections.get(&ManifestSection
                                {
                                    identifier: obj_name.clone(),
                                    index: input_section_idx
                                })
                                {
                                    Some(section) => *section,
                                    None => fatal_msg!("Unexpected error: Can't map input section {:?}:{} to output section", obj_name, input_section_idx.0)
                                };
                                elf.section_symbol(output_section_idx)
                            },
                            _ => fatal_msg!("Unsupported relocation {:?} in section {:?}:{}", relocation, obj_name, input_section.index().0),
                        };

                        let output_relocation = object::write::Relocation {
                            offset,
                            size: relocation.size(),
                            kind: relocation.kind(),
                            encoding: relocation.encoding(),
                            symbol: output_symbol,
                            addend: relocation.addend(),
                        };

                        if let Err(reason) = elf.add_relocation(output_section, output_relocation)
                        {
                            fatal_msg!("Can't add relocation {:?} in {:?}:{}: {:?}", relocation, obj_name, input_section.index().0, reason)
                        }
                    }
                }
            }
        }
    }
}

/* copy the symbols in the manifest's object files into the executable.
   return a hash table of the executable's symbols */
pub fn symbols(elf: &mut object::write::Object, manifest: &Manifest, output_sections: &HashMap<ManifestSection, SectionId>) -> HashMap<ManifestSymbol, SymbolId>
{
    /* keep track of the symbols in the executable and where they came from */
    let mut output_symbols = HashMap::new();

    /* spin through the symbols in each of the objects in the manifest */
    for (obj_name, mapping) in manifest.raw_objects()
    {
        let parsed = manifest::parse(mapping);
        for symbol in parsed.symbols()
        {
            match symbol.kind()
            {
                SymbolKind::Null | SymbolKind::File => (), /* skip symbols we're not interested in */
                _ =>
                {
                    /* map this symbol in its origin section to its section within the final executable if possible
                       or otherwise copy it across */
                    let (section_type, address) = match symbol.section()
                    {
                        SymbolSection::Section(section_index) =>
                        {
                            /* describe the source file + section for this symbol */
                            let input_section = ManifestSection
                            {
                                identifier: obj_name.clone(),
                                index: section_index
                            };

                            if let Some(output_section_id) = output_sections.get(&input_section)
                            {
                                let section_address = match parsed.section_by_index(section_index)
                                {
                                    Ok(section) => section.address(),
                                    Err(reason) => fatal_msg!("Unexpected error: Section index {:?} not found in {:?}: {}", section_index, obj_name, reason)
                                };
                                
                                (object::write::SymbolSection::Section(*output_section_id), symbol.address() - section_address)
                            }
                            else
                            {
                                /* bail out of this loop iteration if its section didn't make it into the executable */
                                continue;                                
                            }
                        },

                        /* map section read types to write types */
                        SymbolSection::None      => (object::write::SymbolSection::None, symbol.address()),
                        SymbolSection::Undefined => (object::write::SymbolSection::Undefined, symbol.address()),
                        SymbolSection::Absolute  => (object::write::SymbolSection::Absolute, symbol.address()),
                        SymbolSection::Common    => (object::write::SymbolSection::Common, symbol.address()),
                        section_type => fatal_msg!("Unexpected symbol section {:?} in {:?}", section_type, obj_name)
                    };

                    let symbol_flags: SymbolFlags<SectionId> = match symbol.flags()
                    {
                        SymbolFlags::None => SymbolFlags::None,
                        SymbolFlags::Elf { st_info, st_other } => SymbolFlags::Elf { st_info, st_other },
                        other => fatal_msg!("Symbol {:?} unsupported in {:?}", other, obj_name)
                    };

                    let output_symbol_name = match symbol.name_bytes()
                    {
                        Ok(bytes) => Vec::from(bytes),
                        Err(reason) => fatal_msg!("Can't read name of symbol {:?} in {:?}: {}", symbol.index(), obj_name, reason)
                    };
                    
                    /* create the new symbol in the target executable */
                    let output_symbol = object::write::Symbol
                    {
                        name: output_symbol_name,
                        value: address,
                        size: symbol.size(),
                        kind: symbol.kind(),
                        scope: symbol.scope(),
                        weak: symbol.is_weak(),
                        section: section_type,
                        flags: symbol_flags
                    };

                    let new_symbol_id = elf.add_symbol(output_symbol);
                    output_symbols.insert(ManifestSymbol { identifier: obj_name.clone(), index: symbol.index() }, new_symbol_id);
                }
            }
        }
    }

    output_symbols
}

/* copy the sections from the manifest's object files into the executable.
   this also updates the elf's e_flags from the object files used
   return a hash table mapping sections in the manifest to sections in the final executable */
pub fn sections<'a, 'b: 'a>(config: &Config, elf: &'a mut object::write::Object<'b>, manifest: &'b Manifest) -> HashMap<ManifestSection, SectionId>
{
    /* keep track of sections created in the executable and where they came from */
    let mut output_sections = HashMap::new();

    /* the sections listed in the link configuration are grouped into blocks of
       standard sections (eg, text, data, etc). locate the blocks of standard
       sections in the configuration, and then copy the sections it specifies
       from the input objects to the final executable. eg, the config file could include:
       
       [section.text]
       include = [ ".entry*", ".text*", ".rodata*" ]
       alignment = 8

       the standard section block is 'text' and the sections it specifies are:
       .entry*, .text*, .rodata*

       therefore, we find the text group in the config file and copy any sections
       matching the group's sections from the input files to the final executable
    */
    for standard_section in SECTIONS.iter()
    {
        if let Some(section_group) = config.get_sections().get(*standard_section)
        {
            for section_to_include in section_group.get_sections_to_include().iter()
            {
                let pattern = WildMatch::new(section_to_include);

                /* spin through the manifest's memory-mapped objects for matching sections to add */
                for (obj_name, mapping) in manifest.raw_objects()
                {
                    let mut flags_updated = false;
                    let parsed = manifest::parse(mapping);

                    /* TODO: support comdats? */
                    if parsed.comdats().count() > 0
                    {
                        fatal_msg!("Unsupported {} comdat(s) in {:?}", parsed.comdats().count(), obj_name);
                    }

                    for section in parsed.sections()
                    {
                        let name = match section.name()
                        {
                            Ok(name) => name,
                            Err(reason) =>
                                fatal_msg!("Can't read section's name in {}: {}",
                                obj_name.to_str().unwrap(), reason)
                        };
                        let kind = section.kind();

                        if pattern.matches(name) && kind != object::SectionKind::Metadata
                        {
                            let data = match section.data()
                            {
                                Ok(bytes) => bytes,
                                Err(reason) =>
                                    fatal_msg!("Can't access section's contents in {}: {}",
                                    obj_name.to_str().unwrap(), reason)
                            };

                            let segment = match section.segment_name()
                            {
                                Ok(name) => name.unwrap_or(""),
                                Err(reason) =>
                                    fatal_msg!("Can't read segment name for {} in {}: {}",
                                    name, obj_name.to_str().unwrap(), reason)
                            };

                            let new_section_id = elf.add_section(Vec::from(segment), Vec::from(name), section.kind());
                            let mut new_section = elf.section_mut(new_section_id);

                            match new_section.is_bss()
                            {
                                true => { new_section.append_bss(section.size(), section.align()); },
                                false => new_section.set_data(data, section.align())
                            };
                            new_section.flags = section.flags();
                            output_sections.insert(ManifestSection { identifier: obj_name.clone(), index: section.index() }, new_section_id);
                            
                            if flags_updated == false
                            {
                                update_flags(elf, parsed.flags());
                                flags_updated = true;
                            }
                        }
                    }
                }
            }
        }
    }

    output_sections
}

/* define e_flags bit position meanings */
const EF_RVC: u32 = 0;                  /* bit    0 = C ext (compressed instructions) in use */
const EF_FLOAT_ABI: u32 = 1;            /* bits 1-2 = float ABI level */
const EF_FLOAT_ABI_MASK: u32 = 0b11;
const EF_FLOAT_ABI_MASK_SHIFTED: u32 = EF_FLOAT_ABI_MASK << EF_FLOAT_ABI;
const EF_RVE: u32 = 3;                  /* bit    3 = RISC-V EABI in use */
const EF_TSO: u32 = 4;                  /* bit    4 = RVTSO memory consistency model required */

/* summarize usage flag bitmask in e_flags */
const EF_USAGE_FLAGS: u32 = (1 << EF_RVC) | (1 << EF_RVE) | (1 << EF_TSO);
 
/* update the elf's e_flags from the given object's flags.
   these flags are processor architecture (RISC-V) dependent and are
   defined here: https://github.com/riscv-non-isa/riscv-elf-psabi-doc */
fn update_flags(elf: &mut object::write::Object, obj_flags: object::FileFlags)
{
    let obj_flags = match obj_flags
    {
        object::FileFlags::None => 0,
        object::FileFlags::Elf { e_flags } => e_flags,
        other => fatal_msg!("Unexpected error: unrecognized object flags {:?}", other)
    };

    let mut elf_flags = match elf.flags
    {
        object::FileFlags::None => 0,
        object::FileFlags::Elf { e_flags } => e_flags,
        other => fatal_msg!("Unexpected error: unrecognized ELF flags {:?}", other)
    };

    /* set bits for compressed instruction, EABI, RVTSO memory model, etc usage */
    elf_flags |= obj_flags & EF_USAGE_FLAGS;

    /* set the floating-point ABI level. each level is backwards compatible.
       ensure the elf's flags reflect the highest level used by its objects */
    let obj_float_abi_level = obj_flags & EF_FLOAT_ABI_MASK_SHIFTED;
    let elf_float_abi_level = elf_flags & EF_FLOAT_ABI_MASK_SHIFTED;
    if obj_float_abi_level > elf_float_abi_level
    {
        elf_flags = obj_float_abi_level | (elf_flags & !EF_FLOAT_ABI_MASK_SHIFTED);
    }

    /* overwrite the elf's flags with the updated flag bits */
    elf.flags = object::FileFlags::Elf { e_flags: elf_flags };
}
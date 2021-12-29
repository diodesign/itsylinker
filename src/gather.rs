/* gather up the sections and symbols in objects to link
 *
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::manifest::{ self, Manifest, FileIdentifier };
use super::config::Config;

use wildmatch::WildMatch;
use indexmap::set::IndexSet;
use object::{ Object, ObjectSection, SectionIndex };

pub const STANDARD_SECTIONS: [(&str, SectionSegment); 4] =
[
    ("text",   SectionSegment::LoadableReadExec),
    ("rodata", SectionSegment::LoadableRead),
    ("data",   SectionSegment::LoadableReadWrite),
    ("bss",    SectionSegment::LoadableReadWrite)
];

/* describe a segment into which sections are grouped */
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum SectionSegment
{
    LoadableRead,
    LoadableReadWrite,
    LoadableReadExec
}

/* describe a section within an object within the manifest */
#[derive(PartialEq, Eq, Hash)]
pub struct ManifestSection
{
    pub identifier: FileIdentifier,
    pub index: SectionIndex,
    pub parent: usize
}

/* describe the gathered up components */
pub struct Collection
{
    sections: IndexSet<ManifestSection>,
    merged: Vec<Vec<usize>>,
    e_flags: object::FileFlags
}

impl Collection
{
    /* collect up the required sections and symbols given the manifest and configuration */
    pub fn new(config: &Config, manifest: &Manifest) -> Collection
    {
        /* keep track of sections, symbols, and flags we're interested in.
           preserve insertion order as that's important for sections at least */
        let mut sections = IndexSet::new();
        let mut e_flags = object::FileFlags::None;

        /* the link configuration file groups sections to include into
           blocks of standard sections (text, rodata, data, bss). iterate over
           the standard sections in the config, scanning the manifest's object files
           for sections that match the sections specified in the block */
        for standard_section_idx in 0..STANDARD_SECTIONS.len()
        {
            let standard_section = STANDARD_SECTIONS[standard_section_idx].0;

            if let Some(section_group) = config.get_sections().get(standard_section)
            {
                for section_to_include in section_group.get_sections_to_include().iter()
                {
                    let pattern = WildMatch::new(section_to_include);

                    /* spin through the memory-mapped object files in the manifest and
                       their sections for matching sections to include */
                    for (obj_name, mapping) in manifest.raw_objects()
                    {
                        let mut flags_updated = false;
                        let parsed = manifest::parse(mapping);

                        /* TODO: support comdats? */
                        if parsed.comdats().count() > 0
                        {
                            fatal_msg!("Unsupported {} comdat(s) sections in {:?}", parsed.comdats().count(), obj_name);
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

                            /* does the section match the section name we're interested in? */
                            if pattern.matches(name) && kind != object::SectionKind::Metadata
                            {
                                /* if so, try to insert it */
                                if sections.insert(ManifestSection
                                {
                                    identifier: obj_name.to_path_buf(),
                                    index: section.index(),
                                    parent: standard_section_idx
                                })
                                {                                    
                                    /* if we're here then the insertion was successful.
                                       update the e_flags once per object file */
                                    if flags_updated == false
                                    {
                                        e_flags = update_e_flags(e_flags, parsed.flags());
                                        flags_updated = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Collection
        {
            sections,
            e_flags,
            merged:
            {
                /* initialize array of standard section groups with empty queues */
                let mut list: Vec<Vec<usize>> = Vec::new();
                for _ in 0..STANDARD_SECTIONS.len()
                {
                    list.push(Vec::new());
                }
                list
            }
        }
    }

    /* merge sections into standard sections, maintaining order */
    pub fn merge(&mut self)
    {
        /* the merged sections are really just arrays of indices, mapping
           sections in self.sections to standard section groups */
        for section_idx in 0..self.sections.len()
        {
            self.merged[self.sections[section_idx].parent].push(section_idx);
        }
    }

    /* arrange the merged sections into memory */
    pub fn arrange(&self, manifest: &Manifest)
    {
        for standard_section_idx in 0..self.merged.len()
        {
            eprintln!("standard section: .{}:", STANDARD_SECTIONS[standard_section_idx].0);
            let standard_section = &self.merged[standard_section_idx];
            for merged_section_idx in 0..standard_section.len()
            {
                let section_idx = standard_section[merged_section_idx];

                let mapping = match manifest.get(&self.sections[section_idx].identifier)
                {
                    None => fatal_msg!("Can't retrieve file {:?}", self.sections[section_idx].identifier),
                    Some(mapping) => mapping
                };
                
                let parsed = manifest::parse(mapping);
                eprintln!("  {}", parsed.section_by_index(self.sections[section_idx].index).unwrap().name().unwrap_or(""));
            }
        }
    }
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
 
/* update the given ELF file e_flags with the given object file e_flags.
   these flags are processor architecture (RISC-V) dependent and are
   defined here: https://github.com/riscv-non-isa/riscv-elf-psabi-doc */
fn update_e_flags(current_flags: object::FileFlags, obj_flags: object::FileFlags) -> object::FileFlags
{
    let obj_flags = match obj_flags
    {
        object::FileFlags::None => 0,
        object::FileFlags::Elf { e_flags } => e_flags,
        other => fatal_msg!("Unexpected error: unrecognized object flags {:?}", other)
    };

    let mut elf_flags = match current_flags
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

    /* return the updated flag bits */
    object::FileFlags::Elf { e_flags: elf_flags }
}
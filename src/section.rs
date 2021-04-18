/* organize sections in memory
 * 
 * (c) Chris Williams, 2021.
 *
 * See LICENSE for usage and copying.
 */

use super::config::{Config, SECTIONS};
use super::generate::{Executable, SectionLayout};
use wildmatch::WildMatch;

pub fn arrange(config: &Config, exe: &mut Executable)
{
    /* get the major sections defined by the executable */
    let exe_major_sections = config.get_sections();

    /* keep track of section placement */
    let mut offset = 0;

    /* go through the list of major sections we're actually interested in */
    for allowed_major_section in SECTIONS.iter()
    {
        /* get the major section from the executable's config, if present */
        if let Some(major_section) = exe_major_sections.get(*allowed_major_section)
        {
            offset = align_up_to(offset, major_section.get_alignment());
            
            /* finally drill down to the section headers to include */
            for section in major_section.get_sections_to_include().iter()
            {   
                let pattern = WildMatch::new(section);
                let mut to_place: Vec<SectionLayout> = Vec::new();

                /* perform a linear search. given a modest program with 20000 section headers and
                   a per-iteration match time of 600ns, it'll take 12ms to check all section headers once.
                   with three major sections (text, data, bss), we're looking at 36ms in total.
                   if this starts to be a problem, TODO: switch from linear search */
                for (sh_name, sh_list) in exe.iter_section_headers()
                {
                    if pattern.matches(sh_name.as_str())
                    {
                        for sh_src in sh_list
                        {
                            to_place.push(SectionLayout::new(offset, sh_src.get_filename().clone(), sh_src.get_sh().clone()));
                            offset = offset + sh_src.get_sh().sh_size as usize;
                            offset = align_up_to(offset, major_section.get_alignment());
                        }
                    }
                }

                /* drain to_place into the executable */
                for placement in to_place
                {
                    exe.add_to_layout(placement);
                }
            }
        }
    }
}

/* align value up to nearest alignment-number of bytes.
   note: alignment must be a non-zero power-of-2. ie, 1, 2, 4, 8, 16... */
fn align_up_to(value: usize, alignment: usize) -> usize
{
    let align_down = value & !(alignment-1);
    
    if align_down == value
    {
        value
    }
    else
    {
        align_down + alignment
    }
}
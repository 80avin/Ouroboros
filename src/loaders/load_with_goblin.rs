use crate::memory::{navigation::Section, Memory};
use crate::memory::LiteralState;
use crate::tab_viewer::TabSignals;
use goblin::Object;

pub fn load<'s>(
    bytes: &'s [u8],
    memory: &mut Memory,
    signals: &mut TabSignals,
) -> Result<String, super::LoaderError> {
    let o = Object::parse(&bytes)?;
    let sleigh_lang_id = match o {
        Object::Elf(ref elf) => {
            // Detect architecture from ELF header
            use goblin::elf::header::*;
            match elf.header.e_machine {
                EM_386 => "x86:LE:32:default",
                EM_X86_64 => "x86:LE:64:default",
                _ => {
                    return Err(super::LoaderError::MalformedFile(format!(
                        "Unsupported architecture: e_machine = {}. Only x86 (EM_386) and x86-64 (EM_X86_64) are currently supported.",
                        elf.header.e_machine
                    )));
                }
            }
        }
        Object::PE(ref pe) => {
            // Detect architecture from PE header
            use goblin::pe::header::COFF_MACHINE_X86;
            use goblin::pe::header::COFF_MACHINE_X86_64;
            match pe.header.coff_header.machine {
                COFF_MACHINE_X86 => "x86:LE:32:default",
                COFF_MACHINE_X86_64 => "x86:LE:64:default",
                _ => {
                    return Err(super::LoaderError::MalformedFile(format!(
                        "Unsupported architecture: machine = {:#x}. Only x86 (I386) and x86-64 (AMD64) are currently supported.",
                        pe.header.coff_header.machine
                    )));
                }
            }
        }
        _ => {
            return Err(super::LoaderError::MalformedFile(
                "Unsupported binary format. Only ELF and PE formats are currently supported.".into()
            ));
        }
    };

    match o {
        Object::Elf(elf) => {
            // Load PLT symbols by parsing .rela.plt relocations
            // Find the .plt.sec section first (modern binaries), fallback to .plt
            let plt_section = elf.section_headers.iter()
                .find(|sh| {
                    if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
                        name == ".plt.sec"
                    } else {
                        false
                    }
                })
                .or_else(|| {
                    elf.section_headers.iter()
                        .find(|sh| {
                            if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
                                name == ".plt"
                            } else {
                                false
                            }
                        })
                });

            if let Some(plt_sec) = plt_section {
                let plt_base = plt_sec.sh_addr;

                // Each PLT entry is typically 16 bytes in modern x86-64
                const PLT_ENTRY_SIZE: u64 = 16;

                // Iterate over PLT relocations
                for (index, reloc) in elf.pltrelocs.iter().enumerate() {
                    // Calculate PLT stub address: base + (index * entry_size)
                    let plt_addr = plt_base + (index as u64 * PLT_ENTRY_SIZE);

                    // Get the symbol for this PLT entry
                    if let Some(sym) = elf.dynsyms.get(reloc.r_sym) {
                        if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                            // Add to symbol table with demangled name
                            let demangled = cpp_demangle::Symbol::new(name)
                                .ok()
                                .and_then(|s| s.demangle(&Default::default()).ok())
                                .unwrap_or_else(|| name.to_string());

                            println!("PLT symbol at {:#x}: {} ({})", plt_addr, demangled, name);
                            memory.symbols.add(plt_addr, 8, demangled);
                        }
                    }
                }
            }

            for section in &elf.section_headers {
                use goblin::elf::section_header::*;
                // ignore those sections
                // TODO add more sections to ignore or proper load it, like
                // the dynamic sections
                if matches!(section.sh_type, SHT_NULL | SHT_NOTE) {
                    continue;
                }
                if section.sh_addr == 0 || section.sh_size == 0 {
                    // empty section, ignore it
                    continue;
                }
                let section_offset = section.file_range();
                let section_bytes = section_offset
                    .map(|range| {
                        bytes.get(range.clone()).ok_or_else(|| {
                            super::LoaderError::MalformedFile(format!(
                                "Section offsets are outside ({:#X}..{:#X}) of file content size.",
                                range.start, range.end,
                            ))
                        })
                    })
                    .transpose()?;

                let name = elf.shdr_strtab.get_at(section.sh_name).unwrap_or("NoName");
                memory.navigation.sections.push(Section::new(
                    name.into(),
                    section.sh_addr,
                    section.sh_size.try_into().unwrap(),
                ));
                // its possible that section_bytes is smaller then the section size
                // this happen in case the data is unitialized, in this case
                // we just create a buffer with zeros
                let mut section_raw = vec![0u8; section.sh_size.try_into().unwrap()];
                if let Some(section_bytes) = section_bytes {
                    section_raw.copy_from_slice(section_bytes);
                }
                let literal = LiteralState::from_bytes(section.sh_addr, section_raw);
                memory
                    .literal
                    .insert_strict(literal.get_interval(), literal)
                    .unwrap();
            }
            // add the entry points from the ELF
            if elf.entry != 0 {
                println!("Entry point: 0x{:x}", elf.entry);
                signals.define_function(elf.entry);
            }
        }
        Object::PE(pe) => {
            // Load all sections into memory
            // TODO: Figure out header image size
            memory
                .navigation
                .sections
                .push(Section::new("Headers".into(), pe.image_base, 1024));
            for section in pe.sections {
                if let Some(data) = section.data(bytes)? {
                    if memory.navigation.sections.len() == 1 {
                        let bytes = &bytes[0..section.pointer_to_raw_data as usize];
                        let literal = LiteralState::from_bytes(pe.image_base, bytes.to_vec());
                        memory.navigation.sections[0].virtual_size =
                            section.pointer_to_raw_data as usize;
                        memory
                            .literal
                            .insert_strict(literal.get_interval(), literal)
                            .unwrap();
                    }
                    let mut section = Section::from(&section);
                    // if section.virtual_address.0 as u32 == section.pointer_to_raw_data {
                    section.virtual_address.0 += pe.image_base;
                    // }
                    let literal =
                        LiteralState::from_bytes(section.virtual_address, data.into_owned());
                    memory.navigation.sections.push(section);
                    memory
                        .literal
                        .insert_strict(literal.get_interval(), literal)
                        .unwrap();
                } else {
                    return Err(super::LoaderError::MalformedFile(
                        "Section offsets are outside of file content size.".into(),
                    ));
                }
            }
            // pe.image_base
            // for entry in pe.imports {
            //     println!("{entry:?}")
            // }
            // if let Some(data) = pe.import_data {
            //     for entry in data.import_data {
            //         println!("{entry:?}")
            //     }
            // }
            println!("Entry point: 0x{:x}", pe.entry as u64 + pe.image_base);
            signals.define_function(pe.entry as u64 + pe.image_base);
        }
        _ => unreachable!("Architecture detection should have caught unsupported formats"),
    }
    Ok(sleigh_lang_id.to_string())
}

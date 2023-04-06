use std::{
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("tests/text_render");
    let file_data = fs::read(path)?;
    let file_data = file_data.as_slice();

    let elf_file = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(file_data)?;

    let section_headers = elf_file
        .section_headers()
        .expect("Expected section headers");

    let mut output = BufWriter::new(fs::File::create("out.gba")?);

    let mut header = gbafix::GBAHeader::default();

    let mut written_header = false;
    for section_header in section_headers.iter() {
        const SHT_NOBITS: u32 = 8;
        const SHT_NULL: u32 = 0;
        const SHF_ALLOC: u64 = 2;

        if (section_header.sh_type == SHT_NOBITS || section_header.sh_type == SHT_NULL)
            || section_header.sh_flags & SHF_ALLOC == 0
        {
            continue;
        }

        let (mut data, compression) = elf_file.section_data(&section_header)?;
        if let Some(compression) = compression {
            panic!("Cannot decompress elf content, but got compression header {compression:?}");
        }

        if !written_header {
            assert!(
                data.len() > 192,
                "first section must be at least as big as the gba header"
            );

            header.start_code = data[0..4].try_into().unwrap();
            header.update_checksum();

            let header_bytes = bytemuck::bytes_of(&header);
            output.write_all(header_bytes)?;

            data = &data[192..];
            written_header = true;
        }

        output.write_all(data)?;
    }

    output.flush()?;

    Ok(())
}

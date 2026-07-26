use std::{env, fs, io};

fn strip_after_char(input: &str, delimiter: char) -> &str {
    match input.find(delimiter) {
        Some(index) => &input[..index],
        None => input,
    }
}

fn main() -> io::Result<()> {
    let current_dir = env::current_dir()?;

    println!("current dir: {:?}", &current_dir);

    let files = fs::read_dir(current_dir)?.filter_map(|e| e.ok()).filter(|e| {
        let file_name = e.file_name();

        file_name.to_string_lossy().starts_with("nvram_") && file_name.to_string_lossy().ends_with(".txt")
    });

    for file in files {
        let text = fs::read_to_string(file.path())?;
        let mut bytes: Vec<u8> = vec![];

        for line in text
            .lines()
            .map(|line| strip_after_char(line, '#').trim_end())
            .filter(|line| !line.is_empty())
        {
            bytes.extend_from_slice(line.as_bytes());
            bytes.extend_from_slice(&b"\x00"[..]);
        }

        bytes.extend_from_slice(&b"\x00\x00"[..]);

        let target = file.path().with_extension("bin");

        fs::write(&target, bytes)?;
        println!("Wrote {:?}", target.file_name().unwrap_or_default().to_owned());
    }

    Ok(())
}

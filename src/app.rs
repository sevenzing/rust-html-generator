use crate::{
    args::Settings,
    parser,
    render::{
        generate_other_file_html, generate_report, generate_rust_file_html, MyPath, SyntaxProcessor,
    },
};
use std::collections::HashMap;

pub fn run_report_generator(settings: &Settings) -> Result<(), anyhow::Error> {
    let root = settings.dir.clone();
    if !root.is_dir() {
        return Err(anyhow::anyhow!("dir argument is not actual directory"));
    };
    let (host, vfs) = parser::get_analysis(&root, settings.scan_whole)?;
    let files = parser::scan(&root, &vfs)?;
    let processor = SyntaxProcessor::new(host, vfs);
    let filenames: Vec<MyPath> = files
        .values()
        .map(|file_info| MyPath::new(&file_info.file_relative_path))
        .collect();

    let mut files_content = HashMap::new();
    for (file_name, file_info) in files.into_iter() {
        let generated_html = match file_info.ra_file_id {
            Some(file_id) => {
                println!("highlight file {:?}", file_info.file_relative_path);
                let hightlight = processor.process_file(file_id);
                let folding_ranges = processor.get_folding_ranges(file_id);
                generate_rust_file_html(hightlight, folding_ranges, &file_info.content, settings)?
            }
            None => generate_other_file_html(&file_info.content)?,
        };
        files_content.insert(file_name, generated_html);
    }
    let s = generate_report(
        filenames,
        files_content,
        root.file_name().unwrap().to_str().unwrap(),
        settings.no_compress,
    );
    std::fs::write(&settings.output, s).expect("unable to write file");
    Ok(())
}

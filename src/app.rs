use crate::{
    args::Settings,
    parser,
    render::{HtmlGenerator, MyPath, ReportGenerator, SyntaxProcessor},
};
use std::collections::HashMap;

pub fn run_report_generator(settings: &Settings) -> Result<(), anyhow::Error> {
    let root = settings.dir.clone();
    if !root.is_dir() {
        return Err(anyhow::anyhow!("dir argument is not actual directory"));
    };
    let (host, vfs) = parser::get_analysis(&root, settings.scan_whole)?;
    let files = parser::scan(&root, &vfs)?;
    let filenames: Vec<MyPath> = files
        .values()
        .map(|file_info| MyPath::new(&file_info.relative_path))
        .collect();
    let processor = SyntaxProcessor::new(host, vfs);
    let generator = HtmlGenerator::new();
    let report_generator = ReportGenerator::default();

    let files_content: HashMap<String, String> = files
        .into_iter()
        .map(|(file_name, file_info)| {
            generator
                .generate(&processor, file_info, settings)
                .map(|content| (file_name, content))
        })
        .collect::<Result<_, _>>()?;

    let output = report_generator.generate(
        filenames,
        files_content,
        root.file_name().unwrap().to_str().unwrap(),
        settings.no_compress,
    );
    std::fs::write(&settings.output, output).expect("unable to write file");
    Ok(())
}

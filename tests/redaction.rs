use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use std::path::Path;
use std::process::Command;

const SECRET: &str = "000-12-3456";
const LABEL: &str = "Public label";

fn write_pdf(path: &Path, value: &str) {
    let mut doc = Document::with_version("1.4");
    let pages = doc.new_object_id();
    let font = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
    });
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 12.into()]),
            Operation::new("Td", vec![72.into(), 700.into()]),
            Operation::new("Tj", vec![Object::string_literal(LABEL)]),
            Operation::new("Td", vec![0.into(), (-30).into()]),
            Operation::new("Tj", vec![Object::string_literal(value)]),
            Operation::new("ET", vec![]),
        ],
    };
    let contents = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page = doc.add_object(dictionary! {
        "Type" => "Page", "Parent" => pages, "Contents" => contents,
        "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
        "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
    });
    doc.objects.insert(
        pages,
        dictionary! {
            "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1,
        }
        .into(),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages });
    doc.trailer.set("Root", catalog);
    doc.compress();
    doc.save(path).unwrap();
}

#[test]
fn redaction_discards_original_page_stream() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("sample.pdf");
    let output = temp.path().join("output");
    write_pdf(&input, SECRET);
    assert!(Document::load(&input)
        .unwrap()
        .extract_text(&[1])
        .unwrap()
        .contains(SECRET));

    let status = Command::new(env!("CARGO_BIN_EXE_pdfredact"))
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());

    let result = Document::load(output.join("sample.pdf")).unwrap();
    let text = result.extract_text(&[1]).unwrap();
    assert!(text.contains(LABEL));
    assert!(!text.contains(SECRET));
    for object in result.objects.values() {
        if let Object::Stream(stream) = object {
            let data = if stream.dict.has(b"Filter") {
                stream.decompressed_content().unwrap()
            } else {
                stream.content.clone()
            };
            assert!(!data
                .windows(SECRET.len())
                .any(|bytes| bytes == SECRET.as_bytes()));
        }
    }
}

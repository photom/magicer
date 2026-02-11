use magicer::domain::repositories::magic_repository::MagicRepository;
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;

#[tokio::test]
async fn test_analyze_buffer_pdf() {
    let repo = FakeMagicRepository::new().expect("Failed to create FakeMagicRepository");
    let pdf_data = b"%PDF-1.4\n";
    
    let (mime, desc) = repo.analyze_buffer(pdf_data, "test.pdf").await.unwrap();
    
    assert_eq!(mime.as_str(), "application/pdf");
    assert!(desc.contains("PDF"));
}

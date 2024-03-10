use libgen_scraper::{
    book::LibgenBook,
    scraper::{self, LibgenClient},
};

#[test]
fn download_book() {
    let test_client = LibgenClient::new();

    let generic_book = "Python for Security and Networking".to_string();

    // let valid_result = LibgenBook {
    //         libgen_id: 3759134,
    //         libgen_md5: "6bed397b612b9e3994a7dc2d6b5440ba".to_owned(),
    //         file_type: "epub".to_owned(),
    //         title: "Python for Security and Networking: Leverage Python modules and tools in securing your network and applications".to_owned(),
    //         authors: vec!["José Manuel Ortega".to_string()],
    //         publisher: "Packt Publishing".to_owned(),
    //     };
    let result = test_client.search_book_by_title(&generic_book);

    // match tokio::runtime::Runtime::new().unwrap().block_on(result) {
    //     Ok(actual_result) => {
    //         // Assert equality
    //         match actual_result {
    //             Some(result) => {
    //                 //result.download().is_ok();
    //                 match result.download() {
    //                     Ok(res) => assert!(true),
    //                     Err(r) => panic!("{}", r),
    //                 }
    //                 // assert!(result.download().is_ok());
    //             }
    //             None => panic!("search result was None"),
    //         }
    //     }
    //     Err(err) => {
    //         // If search function returns an error, fail the test
    //         panic!("Error occurred during search: {:?}", err);
    //     }
    // }
}

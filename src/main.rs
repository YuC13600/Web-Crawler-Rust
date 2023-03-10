use fantoccini::{Client, Locator};
use std::process::Command;
use colored::*;

#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {

    //setting
    let web_driver_adderss = "http://localhost:4444";
    let domain_name = "https://danbooru.donmai.us";
    let origin_url = String::from("https://danbooru.donmai.us/posts?tags=siesta_%28tantei_wa_mou_shindeiru%29+&z=5");
    let folder_name = String::from("siesta");
    
    let mut failed_case: Vec<String> = Vec::new();
    let a = Client::new(web_driver_adderss).await.expect("client a failed to connect to WebDriver");
    a.goto(origin_url.as_str()).await?;
    let page_1_all_page_tab: Vec<_> = a.find_all(Locator::Css(".paginator-page")).await?;
    let page_quantity_str = page_1_all_page_tab[page_1_all_page_tab.len() - 1].clone().html(true).await.unwrap();
    println!("page quantity: {}", page_quantity_str);
    let page_quantity = page_quantity_str.parse::<i32>().unwrap();
    a.close().await.expect("closing client a failed");
    
    let mut url_of_each_page = vec![String::new(); page_quantity as usize];
    for i in 0..page_quantity {
        url_of_each_page[i as usize].push_str(format!("https://danbooru.donmai.us/posts?page={}&tags=siesta_%28tantei_wa_mou_shindeiru%29+&z=5", i+1).as_str());
    }

    let mut post_pages: Vec<String> = Vec::new();
    for (i, page) in url_of_each_page.iter().enumerate() {
        println!("collocting post url from page {}", i+1);
        let c = Client::new(web_driver_adderss).await.expect("client c failed to connect to WebDriver");
        c.goto(page.as_str()).await?;
        
        let previews: Vec<_> = c.find_all(Locator::Css(".post-preview-link")).await?;
        
        for x in previews.iter() {
            let this_page_html = x.html(false).await.unwrap();
            let mut this_page = this_page_html.trim().split('\"');
            loop {
                let this_part = this_page.next().expect("page url not found");
                if this_part.contains("/posts/") {
                    post_pages.push(format!("{}{}", domain_name, this_part).to_string());
                    break;
                }
            }
        }
        c.close().await.expect("closing client c failed");
    }
    let post_quantity = post_pages.len();
    println!("post quantity: {}", post_quantity);
    
    Command::new("powershell").arg(format!("mkdir {}", folder_name)).output().expect("powershell err");
    for (i, page) in post_pages.iter().enumerate() {
        let d = Client::new(web_driver_adderss).await.expect("client d failed to connect to WebDriver");
        d.goto(page).await?;
        let img_html = match d.find(Locator::Css(".image-view-original-link")).await {
            Ok(img) => img.html(false).await?,
            Err(_) => {
                println!("{} image {} original quality not found\npost url: {}", "[warning]".yellow(), i+1, page);
                match d.find(Locator::Css(".fit-width")).await {
                    Ok(img) => {
                        let this_html = img.html(false).await?;   
                        let mut temp_html = this_html.trim().split('\"');                   
                        let file_url = loop {
                            let this_part = temp_html.next().expect("connot find preview file");
                            if this_part.contains("https://") {
                                break this_part.to_string();
                            }
                        };
                        println!("file url: {}", file_url);
                        let filename_extension = loop {
                            let mut last_point = 0;
                            for (i, c) in file_url.chars().enumerate() {
                                if c == '.' {
                                    last_point = i;
                                }
                            }
                            break (&file_url[last_point..]).to_string();
                        };
                        println!("file type: {}", filename_extension);
                        println!("trying to download file");
                        Command::new("powershell").arg(format!("ffmpeg -i {} ./siesta/siesta_{}{}", file_url, i+1, filename_extension)).output().expect("powershell err");
                        println!("download of this file completed");
                    },
                    Err(x) => {
                        println!("{}\nError Message: {}", "Failed to download file".red(), x);
                        failed_case.push(format!("Case {}", i+1));
                        continue;
                    }
                }
                d.close().await.expect("closing client d failed");
                continue;
            }
        };
        let mut temp_str = img_html.trim().split('\"');
        let img_url = loop {
            let this_part = temp_str.next().expect("img url not found");
            if this_part.contains("https") {
                println!("img url: {}", this_part);
                break this_part.to_string()
            }
        };
        Command::new("powershell").arg(format!("ffmpeg -i {} ./siesta/siesta_{}.jpg", img_url, i)).output().expect("powershell err");
        println!("img {} of {} downloaded", i+1, post_quantity);
        d.close().await.expect("closing client d failed");
    }
    
    println!("{} cases completed", post_pages.len());
    if failed_case.len() != 0 {
        println!("\n\n{}", "[Failed Case]".red());
        for f in failed_case.iter() {
            println!("{}", f);
        }
    }
    Ok(())
}

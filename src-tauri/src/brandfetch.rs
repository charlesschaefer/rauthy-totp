use minreq;
use serde::{Deserialize};


#[derive(Debug, Deserialize)] #[allow(non_snake_case, unused)]
pub struct Brand {
    pub brandId: String,
    pub claimed: bool,
    pub domain: String,
    pub icon: String, // icon url
    pub name: String,
}

pub fn search_brand(name: &str, client_id: &str) -> Result<Vec<Brand>, Box<dyn std::error::Error>> {
    let url = format!("https://api.brandfetch.io/v2/search/{}?c={}", name, client_id);
    
    let response = minreq::get(&url)
        //.add_header("Authorization", &format!("Bearer {}", client_id))
        .send()?;
    if response.status_code == 200 {
        // let body: serde_json::Value = serde_json::from_str(&response.as_str()?)?;
        let body: Vec<Brand> = serde_json::from_str(&response.as_str()?)?;
        // println!("Brand data: {:?}", body);
        Ok(body)
    } else {
        // println!("Error: {}", response.status_code);
        Err(Box::from(format!("Error: {}", response.status_code)))
    }
}

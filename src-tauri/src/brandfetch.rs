use minreq;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, unused)]
pub struct Brand {
    pub brandId: String,
    pub claimed: bool,
    pub domain: String,
    pub icon: String, // icon url
    pub name: String,
}

pub fn search_brand(name: &str, client_id: &str) -> Result<Vec<Brand>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.brandfetch.io/v2/search/{}?c={}",
        name, client_id
    );

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

// Larger icons are discarded: they are stored inside the encrypted services file.
const MAX_ICON_BYTES: usize = 512 * 1024;

/// Downloads an image and returns it as a `data:` URI, so it can be stored
/// with the service and displayed later without depending on the remote URL
/// (Brandfetch icon URLs stop working after a while) or on network access.
pub fn fetch_icon_data_uri(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = minreq::get(url).with_timeout(15).send()?;
    if response.status_code != 200 {
        return Err(Box::from(format!("Error: {}", response.status_code)));
    }

    let content_type = response
        .headers
        .get("content-type")
        .map(|ct| ct.split(';').next().unwrap_or("").trim().to_string())
        .unwrap_or_default();
    if !content_type.starts_with("image/") {
        return Err(Box::from(format!("Unexpected content type: {}", content_type)));
    }

    let bytes = response.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_ICON_BYTES {
        return Err(Box::from(format!("Invalid icon size: {} bytes", bytes.len())));
    }

    Ok(format!(
        "data:{};base64,{}",
        content_type,
        data_encoding::BASE64.encode(bytes)
    ))
}

/// Searches the brand by its name and returns its icon as a `data:` URI,
/// or `None` when no icon could be found or downloaded.
pub fn find_brand_icon(name: &str) -> Option<String> {
    let client_id = env!(
        "BRANDFETCH_USER_ID",
        "Brandfetch user_id env var not defined"
    );

    let brands = match search_brand(name, client_id) {
        Ok(brands) => brands,
        Err(err) => {
            dbg!("Error searching brand logo: {}", err);
            return None;
        }
    };
    let icon_url = &brands.first()?.icon;

    match fetch_icon_data_uri(icon_url) {
        Ok(data_uri) => Some(data_uri),
        Err(err) => {
            dbg!("Error downloading brand logo: {}", err);
            None
        }
    }
}

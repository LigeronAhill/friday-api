use std::collections::HashMap;

use serde::Deserialize;
use tracing::error;

use crate::{AppError, Result};

use super::FetchMap;
#[derive(Clone)]
pub struct Spider {
    ort_user: String,
    ort_pass: String,
    client: reqwest::Client,
}

impl Spider {
    pub fn new(ort_user: String, ort_pass: String) -> Result<Self> {
        let cookie_store = std::sync::Arc::new(reqwest::cookie::Jar::default());
        let mut def_head = reqwest::header::HeaderMap::new();
        let v = reqwest::header::HeaderValue::from_str(
            "https://www.yandex.ru/clck/jsredir?from=yandex.ru;suggest;browser&text=",
        )
        .map_err(|e| AppError::ReqwestError(e.to_string()))?;
        def_head.insert(reqwest::header::REFERER, v);
        let client = reqwest::Client::builder()
        .gzip(true)
        .default_headers(def_head)
        .cookie_store(true)
        .cookie_provider(cookie_store.clone())
        .user_agent("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36")
        .build()?;
        Ok(Spider {
            ort_user,
            ort_pass,
            client,
        })
    }
    async fn ortgraph(&self) -> Result<Vec<Vec<u8>>> {
        const BASE_URI: &str = "https://ortgraph.ru";
        const STOCK: &str = "remains/";
        const AUTH: &str = "auth/";
        const PERSONAL: &str = "/personal/";
        let mut auth_uri = BASE_URI.to_string();
        auth_uri.push('/');
        auth_uri.push_str(AUTH);
        let mut form = HashMap::new();
        form.insert("AUTH_FORM", "Y");
        form.insert("TYPE", "AUTH");
        form.insert("backurl", "/auth/");
        form.insert("USER_LOGIN", &self.ort_user);
        form.insert("USER_PASSWORD", &self.ort_pass);
        form.insert("Login", "Войти");
        let response = self
            .client
            .post(&auth_uri)
            .query(&[("login", "yes")])
            .form(&form)
            .send()
            .await?;
        if response.status() != reqwest::StatusCode::OK {
            let b = response.text().await?;
            tracing::info!("Ошибка при запросе остатков Ортграф:\n{b:#?}");
        }
        let mut stock_uri = BASE_URI.to_string();
        stock_uri.push_str(PERSONAL);
        stock_uri.push_str(STOCK);
        let response = self
            .client
            .get(&stock_uri)
            .query(&[("login", "yes")])
            .send()
            .await?;
        let body = response.text().await?;
        let links = get_links(body);
        let mut files = Vec::new();
        for path in links {
            let mut uri = BASE_URI.to_string();
            uri.push_str(&path);
            let temp_res = self.client.get(&uri).send().await?;
            let bytes = temp_res.bytes().await?.as_ref().to_vec();
            files.push(bytes.to_owned());
        }
        Ok(files)
    }
    async fn vvk(&self) -> Result<Vec<Vec<u8>>> {
        const BASE_URI: &str = "https://disk.yandex.ru/d/1qA555p_DbQiaQ";
        let uri = "https://cloud-api.yandex.net:443/v1/disk/public/resources";
        // let download_uri = "https://cloud-api.yandex.net:443/v1/disk/public/resources/download";
        let mut result = Vec::new();
        let response = self
            .client
            .get(uri)
            .query(&[("public_key", BASE_URI)])
            .send()
            .await?;
        let root: Root = response.json().await?;
        for i in root.embedded.items {
            // info!("Got vvk link: {}", i.file);
            let file = self
                .client
                .get(i.file)
                .send()
                .await?
                .bytes()
                .await?
                .to_vec();
            result.push(file)
        }
        Ok(result)
    }
    async fn sf(&self) -> Result<(Vec<u8>, chrono::DateTime<chrono::Utc>)> {
        use chrono::prelude::*;
        const BASE_URI: &str = "https://cloud.mail.ru/public/SA23/oHuEdQLmS";
        let text = self.client.get(BASE_URI).send().await?.text().await?;
        let weblink_re =
            regex::Regex::new(r#""weblink_get":\S"count":"1","url":"(?<url>\S+/no)"},"#).unwrap();
        let filename_re = regex::Regex::new(
            r#""name":"Остатки СФ на  (?<date>[\d\.]+) Клиентские Ковровые \.xlsx","weblink":"(?<url>[A-zА-я\/\s\d\.]+)","#,
        ).unwrap();
        let today = chrono::Utc::now();
        let result = (Vec::new(), today);
        if let Some(wl_capture) = weblink_re.captures(&text) {
            let weblink = wl_capture.name("url").unwrap().as_str();
            if let Some(filename_capture) = filename_re.captures(&text) {
                let filename = filename_capture.name("url").unwrap().as_str();
                let date_str = filename_capture.name("date").unwrap().as_str();
                let day = date_str
                    .split('.')
                    .collect::<Vec<_>>()
                    .first()
                    .and_then(|w| w.parse::<u32>().ok())
                    .unwrap_or(today.day());
                let month = date_str
                    .split('.')
                    .collect::<Vec<_>>()
                    .get(1)
                    .and_then(|w| w.parse::<u32>().ok())
                    .unwrap_or(today.month());
                let year = date_str
                    .split('.')
                    .collect::<Vec<_>>()
                    .get(2)
                    .and_then(|w| w.parse::<i32>().ok())
                    .unwrap_or(today.year());
                let date = chrono::Utc
                    .with_ymd_and_hms(year, month, day, 0, 0, 0)
                    .unwrap();
                let uri = format!("{weblink}/{filename}");
                let file = self.client.get(&uri).send().await?.bytes().await?.to_vec();
                Ok((file, date))
            } else {
                Ok(result)
            }
        } else {
            Ok(result)
        }
    }
    pub async fn get_web(&self) -> Result<FetchMap> {
        let now = chrono::Utc::now();
        let mut map = HashMap::new();
        if let Ok(ort) = self.ortgraph().await {
            map.insert("ortgraph".to_owned(), (ort, now));
        } else {
            error!("Ошибка получения ortgraph");
        }
        match self.vvk().await {
            Ok(vvk) => {
                map.insert("vvk".to_owned(), (vvk, now));
            }
            Err(e) => error!("{e:?}"),
        }
        if let Ok((sf, received)) = self.sf().await {
            if !sf.is_empty() {
                map.insert("sportflooring".to_owned(), (vec![sf], received));
            } else {
                error!("Ошибка получения sf");
            }
        } else {
            error!("Ошибка получения sf");
        }
        Ok(map)
    }
}

fn get_links(body: String) -> Vec<String> {
    let mut result = Vec::new();
    let dom = tl::parse(&body, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let links = dom.query_selector("a[href]").unwrap();
    for link in links {
        let tag = link.get(parser).unwrap().as_tag().unwrap();
        let l = tag
            .attributes()
            .get("href")
            .flatten()
            .unwrap()
            .as_utf8_str()
            .to_string();
        let annotation = tag.inner_html(parser);
        if l.contains(".xls")
            && l.contains("upload")
            && (annotation.to_lowercase().contains("ковр")
                || annotation.to_lowercase().contains("напол"))
        {
            // info!("Got link for {annotation}: {l}");
            result.push(l)
        }
    }
    result
}
#[derive(Deserialize)]
struct Root {
    #[serde(rename = "_embedded")]
    pub embedded: Embedded,
}
#[derive(Deserialize)]
struct Embedded {
    items: Vec<Item>,
}
#[derive(Deserialize)]
struct Item {
    file: String,
}

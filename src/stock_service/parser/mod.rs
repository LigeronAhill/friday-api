use super::FetchMap;
use crate::models::Stock;

mod carpetland;
mod fancy;
mod fenix;
mod fox;
mod opus;
mod ortgraph;
mod sf;
mod vvk;
mod zefir;

pub async fn parse(fetches: FetchMap) -> Vec<Stock> {
    let mut result = Vec::new();
    for (supplier, (files, received)) in fetches {
        match supplier.as_str() {
            "opus" => {
                let res = opus::parser(files.clone(), received).await;
                result.extend(res);
            }
            "fox" => {
                let res = fox::parser(files.clone(), received).await;
                result.extend(res);
            }
            "fancy" => {
                let res = fancy::parser(files.clone(), received).await;
                result.extend(res);
            }
            "carpetland" => {
                let res = carpetland::parser(files.clone(), received).await;
                result.extend(res);
            }
            "zefir" => {
                let res = zefir::parser(files.clone(), received).await;
                result.extend(res);
            }
            "fenix" => {
                let res = fenix::parser(files.clone(), received).await;
                result.extend(res);
            }
            "vvk" => {
                let res = vvk::parser(files.clone(), received).await;
                result.extend(res);
            }
            "ortgraph" => {
                let res = ortgraph::parser(files.clone(), received).await;
                result.extend(res);
            }
            "sportflooring" => {
                let res = sf::parser(
                    files
                        .clone()
                        .first()
                        .map(|s| s.to_owned())
                        .unwrap_or_default(),
                    received,
                )
                .await;
                result.extend(res);
            }
            _ => continue,
        }
    }
    result
}

pub fn clear_string(input: impl AsRef<str>) -> String {
    input
        .as_ref()
        .split_whitespace()
        .map(|w| w.trim().to_uppercase())
        .collect::<Vec<_>>()
        .join(" ")
}

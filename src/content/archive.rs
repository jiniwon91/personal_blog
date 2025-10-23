use chrono::{Datelike, NaiveDate};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use tera::Context;

use super::posts::{PostRef, PostRefContext};
use crate::{
    content::PostItem, context::RenderContext, item::TeraItem, paths::AbsPath, site_url::SiteUrl,
};

pub fn post_archives(posts: &BTreeMap<PostRef, PostItem>) -> Vec<ArchiveItem> {
    let post_refs: Vec<PostRef> = posts
        .iter()
        .filter_map(|(post_ref, post)| {
            if post.is_draft {
                None
            } else {
                Some(post_ref.clone())
            }
        })
        .collect();

    let mut by_year: HashMap<i32, Vec<PostRef>> = HashMap::new();
    let mut by_year_month: HashMap<(i32, u32), Vec<PostRef>> = HashMap::new();

    for post in post_refs.iter() {
        by_year
            .entry(post.order.created.year())
            .or_default()
            .push(post.clone());

        by_year_month
            .entry((post.order.created.year(), post.order.created.month()))
            .or_default()
            .push(post.clone());
    }

    let favorite = post_refs
        .iter()
        .filter(|post_ref| {
            let post = posts.get(post_ref).unwrap();
            post.favorite
        })
        .map(Clone::clone)
        .collect();

    let mut res = vec![
        ArchiveItem {
            title: "All posts".to_string(),
            url: SiteUrl::parse("/blog").unwrap(),
            posts: post_refs,
            tag_filter: None,
        },
        ArchiveItem {
            title: "Favorite posts".to_string(),
            url: SiteUrl::parse("/favorite").unwrap(),
            posts: favorite,
            tag_filter: None,
        },
    ];
    res.extend(by_year.into_iter().map(|(year, posts)| ArchiveItem {
        title: format!("{}", year),
        url: SiteUrl::parse(&format!("/blog/{}", year)).unwrap(),
        posts,
        tag_filter: None,
    }));
    res.extend(by_year_month.into_iter().map(|((year, month), posts)| {
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        ArchiveItem {
            title: date.format("%B %Y").to_string(),
            url: SiteUrl::parse(&date.format("/blog/%Y/%m").to_string()).unwrap(),
            posts,
            tag_filter: None,
        }
    }));
    res
}

#[derive(Debug)]
pub struct ArchiveItem {
    pub title: String,
    pub url: SiteUrl,
    pub posts: Vec<PostRef>,
    pub tag_filter: Option<String>,
}

impl TeraItem for ArchiveItem {
    fn context(&self, ctx: &RenderContext) -> Context {
        Context::from_serialize(ArchiveContext {
            title: &self.title,
            posts: self
                .posts
                .iter()
                .map(|post| PostRefContext::from_ref(post, ctx))
                .collect(),
            tag_filter: self.tag_filter.clone(),
        })
        .unwrap()
    }

    fn template(&self) -> &str {
        "archive.html"
    }

    fn tera_url(&self) -> &SiteUrl {
        &self.url
    }

    fn tera_source_file(&self) -> Option<&AbsPath> {
        None
    }
}

#[derive(Debug, Clone, Serialize)]
struct ArchiveContext<'a> {
    title: &'a str,
    posts: Vec<PostRefContext<'a>>,
    tag_filter: Option<String>,
}

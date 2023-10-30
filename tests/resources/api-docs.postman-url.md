# BriteVerify API Suite Documentation

The BriteVerify API Suite Documentation is available as a [Postman](https://www.postman.com/)
[Collection](https://learning.postman.com/docs/collections/collections-overview/). At the time
of this writing (2023-04-21), the direct URL for the most current specification is:

```text
https://docs.briteverify.com/api/collections/11411276/SzmjyuQH?versionTag=latest
```

## Obtaining The "Latest" Version

### Background

There is *very likely* a better way to do this, but at the time of this writing
(2023-04-23) there doesn't appear to be an official method for accomplishing it
in the [Postman documentation](https://learning.postman.com/docs).

The URL for retrieving a published Postman collection appears (from observed behavior)
to use this structure:

```text
{BASE_URL}/{OWNER_ID}/{PUBLISHED_ID}?ENVIRONMENT={ENVIRONMENT_ID}&VERSION_TAG={VERSION_TAG}
```

wherein `BASE_URL` some domain-specific url, `OWNER_ID` is the Postman account id of the
collection's owner, `PUBLISHED_ID` is a collection-specific identifier issued by Postman as
part of publishing the collection, `ENVIRONMENT_ID` is an identifier that points to a specific
"environment" within the collection itself, and `VERSION_TAG` is (as expected) a unique identifier
for the version of the desired collection you wish to retrieve.

### The BriteVerify API Specifically

In the case of the BriteVerify API specifically, these values are present and can be retrieved from
the HTML source of the publicly available documentation page directly. Examining the raw HTML in a
browser or via curl should reveal something like this:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>BriteVerify API Suite Documentation</title>
    <!-- ... -->
    <meta name="ownerId" content="11411276">  <!-- this should¹ always be the `owner_id` for BriteVerify -->
    <meta name="publishedId" content="SzmjyuQH">  <!-- this should¹ always be the BriteVerify API collection -->
    <meta name="versionTagId" content="latest">  <!-- see note #2 -->
    <meta name="environmentUID" content="11411276-cac17b65-f22d-463a-9a66-9e3dee4b37c8">
    <link rel="canonical" href="https://docs.briteverify.com/">
    <!-- ... -->
  </head>
  <body>
  </body>
</html>
```

> **NOTES:**
>   1) "should" implies no guarantees and is used absent of any meaningful
>      knowledge of the internal workings of Postman's collection publishing
>      system
>   2) The actual value of the `versionTagID` should¹ change as the
>      BriteVerify API evolves, but "latest" does appear to be a valid 
>      value (from observed behavior)


### An Example Implementation

Using the pre-RFC [cargo-script](https://internals.rust-lang.org/t/pre-rfc-cargo-script-for-everyone/18639)
feature, a simple Rust implementation for retrieving the direct URL to the latest version of the BriteVerify
API collection might look like this:

```rust
#!/usr/bin/env cargo-eval
//! ```cargo
//! [dependencies]
//! anyhow = "^1"
//! scraper = "^0.16"
//! reqwest = { version = "^0.11", features = ["blocking"] }
//! ```

use anyhow::Result;
use reqwest::blocking as req;
use scraper::{Html, Selector};

fn from_dom(dom: &Html, selector: &Selector) -> Result<String> {
    Ok(dom.select(selector).next().expect("element missing").value().attr("content").expect("no content").to_string())
}

fn main() -> Result<()> {
    let page = req::get("https://docs.briteverify.com")?.text()?;
    let dom = Html::parse_document(&page);
    
    let selectors = (
        Selector::parse(r#"head > meta[name="ownerId"]"#).expect("couldn't build 'owner_id' selector"),  // owner_id
        Selector::parse(r#"head > link[rel="canonical"]"#).expect("couldn't build 'base_url' selector"),  // base url
        Selector::parse(r#"head > meta[name="publishedId"]"#).expect("couldn't build 'published_id' selector"),  // published_id
        Selector::parse(r#"head > meta[name="versionTagId"]"#).expect("couldn't build 'version_tag' selector"),  // version_tag
        Selector::parse(r#"head > meta[name="environmentUID"]"#).expect("couldn't build 'environment_id' selector"),  // environment_id
        
    );
    
    let values = (
        from_dom(&dom, &selectors.0)?,  // owner_id
        from_dom(&dom, &selectors.4)?,  // base_url
        from_dom(&dom, &selectors.1)?,  // published_id
        from_dom(&dom, &selectors.2)?,  // version_tag
        from_dom(&dom, &selectors.3)?,  // environment_id
    );

    let api_spec_url = format!(
        "{base_url}/api/collections/{owner_id}/{published_id}?environment={environment_id}&versionTag={version_tag}",
        base_url=values.1,
        owner_id=values.0,
        version_tag=values.3,
        published_id=values.2,
        environment_id=values.4,
    );
    
    Ok(println!("Latest BriteVerify API Postman Collection URL: {}", api_spec_url))
}
```

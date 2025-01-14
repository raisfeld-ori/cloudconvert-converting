
# Simple cloudconvert package

This is a package for converting a file from one format to another using cloudconvert.
This package is very minimal and uses Cloudconvert and filebin.io APIs in order to convert.

## Usage

```rust
fn main(){
    use dotenv::dotenv;
    let _ = dotenv().ok();
    let converter = Converter::new("YOUR CLOUDCONVER API KEY");
    let link = converter.convert("path/to/file", // example: "C:\\Users\\user\\file.csv" 
    "file format", // the format of the input file. example: "csv". Find all valid formats here: https://api.cloudconvert.com/v2/convert/formats 
    "output format"); // the output format you want
    println!("{}", link.unwrap()); // A link to the new file. example: https://eu-central.storage.cloudconvert.com/tasks/loremipsumloremipsum
}
```

Get Your cloudconvert API key from [here](https://cloudconvert.com/dashboard/api/v2/keys#) (Requires a cloudconvert account)

## Warnings

1. Cloudconvert requires tokens, and uses 1 token per usage of ```converter.convert```
2. Cloudconvert also has a rate limit of 500 requests.
3. This request uses blocking requests from ```request::blocking```, this could result in errors in async programs.
4. No Advanced error handling. If there's any error it is simply returned back to you.

## extra note

This package is NOT a cloudconvert API client. You can find
that [here](https://crates.io/crates/cloudconvert)

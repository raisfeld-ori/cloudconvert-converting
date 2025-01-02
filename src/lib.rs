use std::path::Path;
use reqwest::blocking::{multipart, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Success{
    pub link: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResultFile{
    pub url: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportResult{
    pub files: Vec<ResultFile>
}
#[derive(Debug, Serialize, Deserialize)]
struct ExportData{
    result: ExportResult
}
#[derive(Debug, Serialize, Deserialize)]
struct ExportComplete{
    data: ExportData
}

#[derive(Debug, Serialize, Deserialize)]
struct Data{
    pub id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskComplete{
    data: Data,
}

/**
The base client required for the library.
requires the cloud convert api key.

Get the api key here: https://cloudconvert.com/dashboard/api/v2/keys
(requires an account btw) 
 */
#[derive(Debug, Clone)]
pub struct Converter{
    cloudconvert: String,
    client: Client
}

pub fn upload<P: AsRef<Path>>(path: P) -> Result<Success, Box<dyn std::error::Error>>{
    let client = Client::new();

    let form = multipart::Form::new()
        .file("file", path)?;
    let res = client.post("https://file.io/")
        .multipart(form)
        .header("accept", "application/json")
        .header("Content-Type", "multipart/form-data")
        .send()?
        ;
    match res.status(){
        reqwest::StatusCode::OK => {
            let success: Success = res.json().unwrap();
            return Ok(success);
        },
        _ => {
            return Err("Failed to upload file: ".into());
        },
    }
}

impl Converter{
    /**
    Creates a new instance of Converter. Requires cloudconvert api key
     */
    pub fn new(cloudconvert_api_key: String) -> Converter{
        Converter{
            cloudconvert: cloudconvert_api_key,
            client: Client::new(),
        }
    }
    fn import_url(&self, url: String) -> Result<TaskComplete, Box<dyn std::error::Error>>{
        let res = self.client.post("https://api.cloudconvert.com/v2/import/url")
            .header("Authorization",  format!("Bearer {}", self.cloudconvert))
            .header("Content-Type", "application/json")
            .form(&json!({
                "url": url,
            }))
            .send()?;
        return Ok(res.json()?);
    }
    fn convert_task(&self, task_id: &str, input_format: &str, output_format: &str) -> Result<TaskComplete, Box<dyn std::error::Error>>{
        let res = self.client.post("https://api.cloudconvert.com/v2/convert")
            .header("Authorization",  format!("Bearer {}", self.cloudconvert))
            .header("Content-Type", "application/json")
            .form(&json!({
                "input": task_id,
                "input_format": input_format,
                "output_format": output_format,
            }))
            .send()?
            ;
        return Ok(res.json()?);
    }
    fn wait_export(&self, task_id: &str) -> Result<ExportComplete, Box<dyn std::error::Error>>{
        let res = self.client.get(format!("https://sync.api.cloudconvert.com/v2/tasks/{}", task_id))
            .header("Authorization", format!("Bearer {}", self.cloudconvert))
            .send()?;
        return Ok(res.json()?);
    }
    fn wait_for_task(&self, task_id: &str) -> Result<TaskComplete, Box<dyn std::error::Error>>{
        let res = self.client
        .get(format!("https://sync.api.cloudconvert.com/v2/tasks/{}", task_id))
        .header("Authorization", format!("Bearer {}", self.cloudconvert))
        .send()?;
        Ok(res.json()?)
    }
    fn export_file(&self, task_id: &str) -> Result<TaskComplete, Box<dyn std::error::Error>>{
        let res = self.client.post("https://api.cloudconvert.com/v2/export/url")
            .header("Authorization", format!("Bearer {}", self.cloudconvert))
            .header("Content-Type", "application/json")
            .form(&json!({
                "input": task_id,
            }))
            .send()?;
        return Ok(res.json()?);
    }
    pub fn convert<T: AsRef<Path>>(&self, file: T, input_format: &str, output_format: &str) -> Result<String, Box<dyn std::error::Error>>{
        let file_url = upload(file)?.link;
        let task = self.import_url(file_url)?;
        self.wait_for_task(&task.data.id)?;
        let task = self.convert_task(&task.data.id, input_format, output_format)?;
        self.wait_for_task(&task.data.id)?;
        let task = self.export_file(&task.data.id)?;
        return Ok(self.wait_export(&task.data.id)?.data.result.files[0].url.clone());
    }
}

#[test]
fn test_tokens(){
    use dotenv::dotenv;
    let _ = dotenv().ok();
    let converter = Converter::new(dotenv::var("cloudconvert").unwrap());
    let res = converter.convert(dotenv::var("file").unwrap(), 
    &dotenv::var("input").unwrap(), 
    &dotenv::var("output").unwrap());
    assert!(res.is_ok());
}
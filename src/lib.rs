use std::path::Path;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
struct InternalData{
    pub id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskComplete{
    data: InternalData,
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadLocation {
    data: DataRah,
}
#[derive(Debug, Serialize, Deserialize)]
struct DataRah{
    id: String,
    result: serde_json::Value,
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
    client: Client,
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
    fn upload(&self, file: impl AsRef<Path>) -> Result<String, Box<dyn std::error::Error>>{
        let server = "https://api.cloudconvert.com/v2/import/upload";

        let res = self.client.post(server)
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer ".to_string() + &self.cloudconvert)
            .send()?;
        let res = res.json::<UploadLocation>()?;
        let id = res.data.id;
        let form = res.data.result.get("form").unwrap().clone();
        
        let server = form.get("url").unwrap();
        let parameters = form.get("parameters").unwrap().clone();

        let multiform = reqwest::blocking::multipart::Form::new()
            .text("key", parameters.get("key").unwrap().as_str().unwrap().to_string())
            .text("acl", parameters.get("acl").unwrap().as_str().unwrap().to_string())
            .text("X-Amz-Algorithm", parameters.get("X-Amz-Algorithm").unwrap().as_str().unwrap().to_string())
            .text("X-Amz-Credential", parameters.get("X-Amz-Credential").unwrap().as_str().unwrap().to_string())
            .text("X-Amz-Date", parameters.get("X-Amz-Date").unwrap().as_str().unwrap().to_string())
            .text("X-Amz-Signature", parameters.get("X-Amz-Signature").unwrap().as_str().unwrap().to_string())
            .text("Policy", parameters.get("Policy").unwrap().as_str().unwrap().to_string())
            .text("success_action_status", parameters.get("success_action_status").unwrap().as_str().unwrap().to_string())
            .file("file", file)?;
        
        let _ = self.client.post(server.as_str().unwrap())
            .header("Authorization", format!("Bearer {}", self.cloudconvert))
            .multipart(multiform)
            .send()?;

        return Ok(id);
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
        let id = self.upload(file)?;
        let task = self.convert_task(&id, input_format, output_format)?;
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

use std::collections::HashMap;

//Java record 대응->Rust struct
#[derive(Debug,Clone)]
pub struct OAuthUserInfo{
    pub provider: String,
    pub provider_id: String,
    pub email: Option<String>,
    pub nickname: Option<String>,
}

//지원하지 않는 provider 에러
#[derive(Debug)]
pub struct UnsupportedProviderError;

impl OAuthUserInfo{
    //static OAuthUserInfo of() 대응
    pub fn of(
        registration_id: &str,
        attrs: &HashMap<String, serde_json::Value>,
    )->Result<Self,UnsupportedProviderError>{

        match registration_id{
            "kakao"=>{
                //String.valueOf(attrs.get("id)) 대응
                let provider_id=attrs.get("id")
                    .map(|v|v.to_string())
                    .unwrap_or_default();

                //(Map<String,Object>) attrs.get("kakao_account") 대응
                let account=attrs.get("kakao_account")
                    .and_then(|v|v.as_object());

                //(Map<String,Object>) attrs.get("properties") 대응
                let props=attrs.get("properties")
                    .and_then(|v|v.as_object());

                let email=account.and_then(|a| a.get("email"))
                    .and_then(|v| v.as_str()).map(|s| s.to_string());

                let nickname=props.and_then(|p| p.get("nickname")).and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Ok(Self{
                    provider: "KAKAO".to_string(),
                    provider_id,
                    email,
                    nickname,
                })
            }

            "naver"=>{
                //(Map<String,Object>) attrs.get("response") 대응
                let resp=attrs.get("response")
                    .and_then(|v| v.as_object());

                let provider_id=resp.and_then(|r|r.get("id")).map(|v|v.to_string())
                    .unwrap_or_default();

                let email=resp.and_then(|r| r.get("email"))
                    .and_then(|v| v.as_str()).map(|s| s.to_string());

                let nickname=resp.and_then(|r| r.get("name")).and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(Self{
                    provider: "NAVER".to_string(),
                    provider_id,
                    email,
                    nickname,
                })

            }

            "google"=>{
                let provider_id=attrs.get("sub").map(|v| v.to_string())
                    .unwrap_or_default();

                let email=attrs.get("email").and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let nickname=attrs.get("name").and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(Self{
                    provider: "GOOGLE".to_string(),
                    provider_id,
                    email,
                    nickname,


                })



            }
            _=>Err(UnsupportedProviderError),
        }
    }
}
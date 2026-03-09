use std::collections::HashMap;
use reqwest::Client;
use crate::auth::model::User;
use crate::auth::repository::UserRepository;
use crate::user::repository::UserInfoRepository;
use super::model::{OAuthUserInfo, OAuthPrincipal, UnsupportedProviderError};

#[derive(Clone)]
pub struct OAuthService{
    user_repository: UserRepository,
    user_info_repository: UserInfoRepository,
    http_client: Client,    //카카오/네이버/구글 API 호출용
}

#[derive(Debug)]
pub enum OAuthError{
    UnsupportedProvider,
    ApiCallFailed,
    DbError,
}

impl OAuthService{
    pub fn new(user_repository: UserRepository, user_info_repository: UserInfoRepository)->Self{
        Self{
            user_repository,
            user_info_repository,
            http_client: Client::new(),
        }
    }

    //loadUser() 대응
    pub async fn load_user(
        &self,
        registration_id: &str,  // "kakao", "naver", "google"
        access_token: &str, //OAuth 인증 후 받은 액세스 토큰
        user_type: Option<String>,  //resolveUserType() 대응(세션 대신 파라미터로)
    )-> Result<OAuthPrincipal, OAuthError>{

        //super.loadUser() 대응
        //Spring이 자동으로 하던 API 호출을 직접 구현
        let attrs=self.fetch_user_info(registration_id, access_token).await?;

        //OAuthUserInfo.of() 대응
        let info=OAuthUserInfo::of(registration_id, &attrs).map_err(|_| OAuthError::UnsupportedProvider)?;

        //resolveUserType() 대응
        //Spring은 세션에서 읽었는데 Result는 세션이 없으니까 파라미터로 받음
        let user_type=user_type.unwrap_or_else(|| "USER".to_string());

        //userRepository.findByProviderAndProviderId().orElseGet() 대응
        let user=match self.user_repository.find_by_provider_and_provider_id(&info.provider,&info.provider_id)
            .await
        {
            //DB에 있으면 기존 유저 반환
            Some(existing_user)=>existing_user,

            //DB에 없으면 신규 회원가입
            //orElseGet(()->{...}) 대응
            None=>{
                //User.builder()...build() 대응
                let new_user=User{
                    user_id: None,
                    provider: Some(info.provider.clone()),
                    provider_id: Some(info.provider_id.clone()),
                    user_email: info.email.clone(),
                    user_nickname: info.nickname.clone(),
                    user_type: user_type.clone(),
                    user_password: "OAUTH_USER".to_string(),
                    ..Default::default()
                };

                //userRepository.save() 대응
                let saved_user=self.user_repository.save(new_user).await
                    .map_err(|_| OAuthError::DbError)?;

                //UserInfo 생성 및 저장
                //"STORE".equals(userType)?createForStoreUser:createForOAuthUser 대응
                let user_info=if user_type=="STORE"{
                    UserInfo::create_for_store_user(&saved_user)
                }else{
                    UserInfo::create_for_oauth_user(&saved_user)
                };

                self.user_info_repository.save(user_info).await.map_err(|_| OAuthError::DbError)?;

                saved_user
            }
        };
        //return new OAuthPrincipal(user, OAuth2User.getAttributes()) 대응
        Ok(OAuthPrincipal::new(user,attrs))
    }
    //super.loadUser() 내부 동작 대응
    //Spring이 자동으로 하던 API 호출을 직접 구현
    async fn fetch_user_info(
        &self,
        registration_id: &str,
        access_token: &str,
    )->Result<HashMap<String,serde_json::Value>,OAuthError>{

        //각 provider별 유저 정보 API URL
        let url=match registration_id{
            "kakao"=>"https://kapi.kakao.com/v2/user/me",
            "naver"=>"https://openapi.naver.com/v1/nid/me",
            "google"=>"https://www.googleapis.com/oauth2/v3/userinfo",
            _=>return Err(OAuthError::UnsupportedProvider),
        };

        //Authorization: Bearer{access_token} 헤더로 API 호출
        let response=self.http_client
            .get(url)
            .bearer_with(access_token)
            .send()
            .await
            .map_err(|_|OAuthError::ApiCallFailed)?
            .json::<HashMap<String,serde_json::Value>>()
            .await
            .map_err(|_|OAuthError::ApiCallFailed)?;

        Ok(response)
    }

}
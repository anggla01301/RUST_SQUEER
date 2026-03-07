use crate::auth::jwt::JwtUtil;
use crate::auth::model::{
    LoginRequestDto, LoginResponseDto, StoreSignUpRequestDto, User, UserInfo, UserSignUpRequestDto,
};
use crate::auth::repository::UserRepository;
use bcrypt::{hash, verify, DEFAULT_COST};

//에러 코드 대응 (CustomException + ErrorCode)

#[derive(Debug)]
pub enum AuthError {
    EmailDuplicated,         //EMAIL_DUPLICATED
    SocialEmailUpDuplicated, //Social_Email_DUPLICATED
    EmailNotFound,           //EMAIL_NOT_FOUND
    PasswordNotMatch,        //PASSWORD_NOT_MATCH
    AccountWithdraw,         //ACCOUNT_WITHDRAW
    UserNotFound,            //USER_NOT_FOUND
    UserInfoNotFound,        //USER_INFO_NOT_FOUND
}

//@Service 대응
pub struct AuthService {
    user_repository: UserRepository,
    jwt_util: JwtUtil,
}

impl AuthService {
    //생성자 주입 대응
    pub fn new(user_repository: UserRepository, jwt_util: JwtUtil) -> Self {
        Self {
            user_repository,
            jwt_util: JwtUtil::new(),
        }
    }
    //signUpUser()대응 (@Transactional)
    pub async fn sign_up_user(&self, dto: UserSignUpRequestDto) -> Result<(), AuthError> {
        //이메일 중복 체크
        if self
            .user_repository
            .exists_by_user_email(&dto.user_email)
            .await
        {
            return Err(AuthError::EmailDuplicated);
        }
        //소셜 이메일 중복 체크
        if self
            .user_repository
            .exists_by_email_and_provider_not_null(&dto.user_email)
            .await
        {
            return Err(AuthError::SocialEmailUpDuplicated);
        }
        //비밀번호 암호화(BCryptPasswordEncoder 대응)
        let hashed_password = hash(&dto.user_password, DEFAULT_COST).unwrap();

        //User 생성(User.builder() 대응)
        let mut user = User::new(dto.user_nickname, hashed_password, "USER".to_string());
        user.user_email = Some(dto.user_email);
        user.user_number = Some(dto.user_number);

        //userRepository.save()대응
        let saved_user = self
            .user_repository
            .save(&user)
            .await
            .ok_or(AuthError::UserNotFound)?;
        //UserInfo 생성(UserInfo.createForNormalUser() 대응)
        let user_id = saved_user.user_id.unwrap();
        let user_info = UserInfo::for_normal_user(user_id);
        //userInfoRepository.save()->나중에 UserInfoRepository 추가 후 연결
        Ok(())
    }

    //signUpStore() 대응
    pub async fn sign_up_store(&self, dto: StoreSignUpRequestDto) -> Result<(), AuthError> {
        if self
            .user_repository
            .exists_by_user_email(&dto.user_email)
            .await
        {
            return Err(AuthError::EmailDuplicated);
        }

        let hashed_password = hash(&dto.user_password, DEFAULT_COST).unwrap();

        let mut user = User::new(dto.user_nickname, hashed_password, "STORE".to_string());
        user.user_email = Some(dto.user_email);
        user.user_number = Some(dto.user_number);

        let saved_user = self
            .user_repository
            .save(&user)
            .await
            .ok_or(AuthError::UserNotFound)?;
        let user_id = saved_user.user_id.unwrap();
        let user_info = UserInfo::for_store_user(user_id);
        //StoreRepository 추가 후 store 저장 연결 예정

        Ok(())
    }

    //login() 대응
    pub async fn login(&self, dto: LoginRequestDto) -> Result<LoginResponseDto, AuthError> {
        //findByUserEmail().orElseThrow() 대응
        let user = self
            .user_repository
            .find_by_user_email(&dto.user_email)
            .await
            .ok_or(AuthError::EmailNotFound)?; //orElseThrow() 대응
                                               //passwordEncoder.matches() 대응
        let password_match = verify(&dto.user_password, &user.user_password).unwrap_or(false);
        if !password_match {
            return Err(AuthError::PasswordNotMatch);
        }
        //탈퇴 여부 체크
        if user.user_is_active == "N" {
            return Err(AuthError::AccountWithdraw);
        }

        let user_id = user.user_id.unwrap();

        //토큰 생성
        let access_token = self.jwt_util.generate_token(
            user_id,
            user.user_email.as_deref().unwrap_or(""),
            &user.user_type,
        );
        let refresh_token = self.jwt_util.generate_refresh_token(user_id);

        //refreshToekn DB에 저장
        self.user_repository
            .update_refresh_token(user_id, &refresh_token)
            .await;

        //firstLogin 체크("PENDING" 타입이면 첫 로그인)
        let first_login = user.user_type == "PENDING";

        Ok(LoginResponseDto {
            token: access_token,
            user_id,
            user_nickname: user.user_nickname.clone(),
            user_email: user.user_email.unwrap_or_default(),
            user_type: user.user_type.clone(),
            user_info_level: 1, //UserInfoRepository 연결 후 수정
            user_info_point: 0, //UserInfoRepository 연결 후 수정
            refresh_token,
        })
    }

    //logout() 대응
    pub async fn logout(&self, user_id: i64) -> Result<(), AuthError> {
        //findById().orElseThrow() 대응
        self.user_repository
            .find_by_id(user_id)
            .await
            .ok_or(AuthError::UserNotFound)?;
        //refreshToken null 처리
        self.user_repository.update_refresh_token(user_id, "").await;

        Ok(())
    }
    //withdraw() 대응
    pub async fn withdraw(&self, user_id:i64)->Result<(),AuthError>{
        let user=self.user_repository.find_by_id(user_id).await.ok_or(AuthError::UserNotFound)?;
        Ok(())
    }



    //updateUserType() 대응
    pub async fn update_user_type(&self, user_id: i64, user_type: &str) -> Result<(), AuthError> {
        self.user_repository
            .find_by_id(user_id)
            .await
            .ok_or(AuthError::UserNotFound)?;

        self.user_repository
            .update_user_type(user_id, user_type)
            .await;

        Ok(())
    }
}

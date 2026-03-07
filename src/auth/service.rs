use bcrypt::{hash,verify,DEFAULT_COST};
use crate::auth::model::{
    User, UserInfo, LoginRequestDto, LoginResponseDto,
    UserSignUpRequestDto,StoreSignUpRequestDto,
};
use crate::auth::repository::UserRepository;
use crate::auth::jwt::JwtUtil;

//에러 코드 대응 (CustomException + ErrorCode)

#[derive(Debug)]
pub enum AuthError{
    EmailDuplicate, //EMAIL_DUPLICATED
    SocialEmailUpDuplicated, //Social_Email_DUPLICATED
    EmailNotFound,  //EMAIL_NOT_FOUND
    PasswordNotMatch, //PASSWORD_NOT_MATCH
    AccountWithdraw, //ACCOUNT_WITHDRAW
    UserNotFound,   //USER_NOT_FOUND
    UserInfoNotFound, //USER_INFO_NOT_FOUND

}

//@Service 대응
pub struct AuthService{
    user_repository: UserRepository,
    jwt_util: JwtUtil,
}

impl AuthService{
    //생성자 주입 대응
    pub fn new(user_repository: UserRepository, jwt_util: JwtUtil)->Self{
        Self{
            user_repository,
            jwt_util: JwtUtil::new(),

        }
    }
    //signUpUser()대응 (@Transactional)
    pub async fn sign_up_user(&self, dto:UserSignUpRequestDto)->Result<(),AuthError>{
        //이메일 중복 체크
        if self.user_repository.exists_by_user_email(&dto.user_email).await{
            return Err(AuthError::EmailDuplicated);
        }
        //소셜 이메일 중복 체크
        if self.user_repository.exists_by_email_and_provider_not_null(&dto.user_email).await{
            return Err(Auth)
        }
    }



}
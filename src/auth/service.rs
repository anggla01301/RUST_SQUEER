use bcrypt::{hash, verify, DEFAULT_COST};
// bcrypt:
//   비밀번호를 평문으로 저장하지 않고 해시해서 저장/검증할 때 쓰는 라이브러리다.
//
// hash(plain, cost):
//   평문 비밀번호를 bcrypt 해시 문자열로 바꾼다.
//
// verify(plain, hashed):
//   입력한 평문 비밀번호가 저장된 해시와 일치하는지 검증한다.
//
// DEFAULT_COST:
//   bcrypt 연산 강도(반복 비용) 기본값이다.
//   숫자가 높을수록 더 안전하지만 더 느리다.

use crate::auth::jwt::JwtUtil;
// JWT 생성/검증 유틸이다.
// access token, refresh token 발급에 사용한다.

use crate::auth::model::{
    LoginRequestDto, LoginResponseDto, StoreSignUpRequestDto, TotalSignUpRequestDto, User,
    UserInfo, UserProfileUpdateDto, UserProfileUpdateResponseDto, UserSignUpRequestDto,
};
// 인증 관련 DTO / 엔티티들이다.
//
// LoginRequestDto:
//   로그인 요청 body
//
// LoginResponseDto:
//   로그인 성공 시 내려줄 응답 DTO
//
// StoreSignUpRequestDto:
//   점주 회원가입 요청 DTO
//
// TotalSignUpRequestDto:
//   통합 회원가입 요청 DTO
//
// User:
//   USER_ 테이블에 대응되는 유저 엔티티
//
// UserInfo:
//   USER_INFO 테이블에 대응되는 엔티티
//
// UserProfileUpdateDto:
//   프로필 수정 요청 DTO
//
// UserProfileUpdateResponseDto:
//   프로필 수정 성공 응답 DTO
//
// UserSignUpRequestDto:
//   일반 사용자 회원가입 요청 DTO

use crate::auth::repository::{UserInfoRepository, UserRepository};
// Repository 계층이다.
// DB 조회/저장/수정 같은 영속성 처리를 담당한다.
//
// UserRepository:
//   USER_ 테이블 담당
//
// UserInfoRepository:
//   USER_INFO 테이블 담당

use crate::store::model::Store;
// STORE_ 테이블에 대응되는 점포 엔티티다.

use crate::store::service::StoreRepository;
// 점포 저장소다.
// 이름은 service에 있지만 역할상 repository처럼 보인다.
// STORE_ 생성/조회 등에 사용한다.

// 인증 비즈니스 로직을 담당하는 서비스 계층이다.
// 자바 스프링으로 치면 @Service 역할이라고 보면 된다.

#[derive(Debug)]
pub enum AuthError {
    // 일반 이메일 회원가입을 하려는데 이미 같은 이메일이 존재하는 경우
    EmailDuplicated,

    // 이미 소셜 계정(provider not null)으로 가입된 이메일인 경우
    // 즉 일반 회원가입 이메일과 소셜 로그인 이메일이 충돌한 경우
    SocialEmailUpDuplicated,

    // 로그인 시 이메일로 유저를 찾지 못한 경우
    EmailNotFound,

    // 비밀번호 검증 실패
    PasswordNotMatch,

    // 탈퇴(비활성화)된 계정인 경우
    AccountWithdraw,

    // user_id 기준으로 유저를 못 찾은 경우
    UserNotFound,

    // USER_INFO를 찾지 못한 경우
    UserInfoNotFound,

    // 회원가입/저장/필수값 누락 등 전반적인 실패를 묶어서 표현하는 에러
    SignupFailed,

    // 허용되지 않은 user_type 값인 경우
    InvalidUserType,
}

#[derive(Clone)]
pub struct AuthService {
    // USER_ 관련 DB 작업 담당
    user_repository: UserRepository,

    // USER_INFO 관련 DB 작업 담당
    user_info_repository: UserInfoRepository,

    // STORE_ 관련 DB 작업 담당
    store_repository: StoreRepository,

    // JWT 발급/검증 유틸
    jwt_util: JwtUtil,
}

impl AuthService {
    // 생성자 역할이다.
    //
    // 자바로 치면 생성자 주입과 비슷하다.
    // repository들과 jwt_util을 받아 AuthService 내부 필드에 저장한다.
    pub fn new(
        user_repository: UserRepository,
        user_info_repository: UserInfoRepository,
        store_repository: StoreRepository,
        jwt_util: JwtUtil,
    ) -> Self {
        Self {
            user_repository,
            user_info_repository,
            store_repository,
            jwt_util,
        }
    }

    // 일반 회원가입
    //
    // 전체 흐름:
    // 1. 소셜 계정 이메일 중복 검사
    // 2. 일반 이메일 중복 검사
    // 3. 비밀번호 해시
    // 4. USER_ 엔티티 생성
    // 5. USER_ 저장
    // 6. 저장된 user_id 추출
    // 7. USER_INFO 초기값 생성/저장
    // 8. 성공 시 Ok(())
    //
    // 반환 타입:
    // Result<(), AuthError>
    //
    // - 성공: Ok(())
    //   여기서 () 는 "의미 있는 반환값 없음"을 뜻한다.
    //
    // - 실패: Err(AuthError::...)
    pub async fn sign_up_user(&self, dto: UserSignUpRequestDto) -> Result<(), AuthError> {
        // 1) "소셜 로그인 계정으로 이미 가입된 이메일"인지 먼저 검사한다.
        //
        // exists_by_email_and_provider_not_null(...)
        //   - 해당 이메일이 존재하고
        //   - provider(카카오/구글/네이버 등)가 null이 아닌 계정이 있는지 확인할 가능성이 크다.
        //
        // .await:
        //   비동기 DB 작업 결과를 기다린다.
        if self
            .user_repository
            .exists_by_email_and_provider_not_null(&dto.user_email)
            .await
        {
            // 이미 소셜 계정 이메일이면 일반 회원가입을 막는다.
            return Err(AuthError::SocialEmailUpDuplicated);
        }

        // 2) 일반 이메일 중복 검사
        //
        // exists_by_user_email(...)
        //   USER_ 테이블에 같은 이메일이 이미 있는지 확인한다.
        if self
            .user_repository
            .exists_by_user_email(&dto.user_email)
            .await
        {
            // 중복이면 즉시 실패 반환
            return Err(AuthError::EmailDuplicated);
        }

        // 3) 비밀번호를 bcrypt 해시로 변환한다.
        //
        // 평문 비밀번호를 DB에 그대로 저장하면 안 되므로 반드시 해시해서 저장해야 한다.
        //
        // unwrap():
        //   hash 실패 시 panic 가능
        //   실무에선 map_err 등으로 서비스 에러 변환하는 게 더 안전할 수 있다.
        let hashed_password = hash(&dto.user_password, DEFAULT_COST).unwrap();

        // 4) USER_ 엔티티를 만든다.
        //
        // User::new(...)는 아마 최소 필수값으로 User 객체를 만드는 생성 함수일 것이다.
        //
        // 전달값:
        // - user_name
        // - user_nickname
        // - hashed_password
        // - "PENDING"
        //
        // 여기서 "PENDING"은 아직 역할(USER/STORE)이 최종 확정되지 않은 상태일 가능성이 크다.
        let mut user = User::new(
            dto.user_name,
            dto.user_nickname,
            hashed_password,
            "PENDING".to_string(),
        );

        // 5) Option 필드로 보이는 값들을 추가 세팅한다.
        //
        // user.user_email = Some(...)
        //   이메일 필드가 Option<String>일 가능성이 높다.
        //
        // Some(...)은 "값이 있음"을 뜻한다.
        user.user_email = Some(dto.user_email);
        user.user_number = Some(dto.user_number);

        // 6) USER_ 저장
        //
        // save(&user).await 의 결과가 아마 Option<User>일 가능성이 크다.
        //
        // ok_or(AuthError::SignupFailed)?
        //   - Some(value)면 value를 꺼내서 계속 진행
        //   - None이면 AuthError::SignupFailed 로 변환해서 즉시 반환
        //
        // ? 연산자:
        //   Err가 나오면 현재 함수에서 바로 return Err(...) 하는 축약 문법이다.
        let saved_user = self
            .user_repository
            .save(&user)
            .await
            .ok_or(AuthError::SignupFailed)?;

        // 7) 저장된 유저에서 user_id 추출
        //
        // DB 저장 후에 PK(user_id)가 채워졌을 가능성이 높다.
        //
        // user_id가 None이면 이상한 상태이므로 UserNotFound 처리
        let user_id = saved_user.user_id.ok_or(AuthError::UserNotFound)?;

        // 8) USER_INFO 초기값 생성 후 저장
        //
        // UserInfo::for_normal_user(user_id)
        //   일반 유저용 기본 초기 상태를 만드는 팩토리 메서드일 가능성이 크다.
        //
        // 예:
        // - 초기 레벨
        // - 초기 포인트
        // - 미션 생성/수행 가능 횟수
        // 등을 세팅했을 수 있다.
        self.user_info_repository
            .save(UserInfo::for_normal_user(user_id))
            .await
            .ok_or(AuthError::SignupFailed)?;

        // 9) 여기까지 모두 성공하면 회원가입 완료
        Ok(())
    }

    // 점주 회원가입
    //
    // 일반 회원가입과 거의 비슷하지만,
    // 추가로:
    // - USER_INFO를 점주용 기본값으로 저장
    // - STORE_ 엔티티도 생성/저장
    //
    // 즉 일반 유저보다 한 단계 더 많은 테이블 작업이 들어간다.
    pub async fn sign_up_store(&self, dto: StoreSignUpRequestDto) -> Result<(), AuthError> {
        // 1) 이메일 중복 검사
        if self
            .user_repository
            .exists_by_user_email(&dto.user_email)
            .await
        {
            return Err(AuthError::EmailDuplicated);
        }

        // 2) 비밀번호 해시
        let hashed_password = hash(&dto.user_password, DEFAULT_COST).unwrap();

        // 3) USER_ 엔티티 생성
        let mut user = User::new(
            dto.user_name,
            dto.user_nickname,
            hashed_password,
            "PENDING".to_string(),
        );
        user.user_email = Some(dto.user_email);
        user.user_number = Some(dto.user_number);

        // 4) USER_ 저장
        let saved_user = self
            .user_repository
            .save(&user)
            .await
            .ok_or(AuthError::SignupFailed)?;

        // 5) 저장된 user_id 추출
        let user_id = saved_user.user_id.ok_or(AuthError::UserNotFound)?;

        // 6) USER_INFO를 점주용 초기값으로 저장
        //
        // 일반 유저와 점주는 초기 상태가 다를 수 있다.
        // 예를 들어:
        // - 미션 생성 가능 횟수
        // - 미션 수행 가능 횟수
        // 등이 다르게 세팅될 수 있다.
        self.user_info_repository
            .save(UserInfo::for_store_user(user_id))
            .await
            .ok_or(AuthError::SignupFailed)?;

        // 7) STORE_ 엔티티 생성 후 저장
        //
        // Store::new(...)
        //   점포명, 카테고리, 위도/경도, user_id를 바탕으로 점포 엔티티 생성
        //
        // 즉 점주 회원가입은 유저만 만드는 게 아니라
        // "이 유저가 소유한 가게 정보"까지 같이 만든다.
        self.store_repository
            .save(&Store::new(
                dto.store_name,
                dto.store_category,
                dto.store_latitude,
                dto.store_longitude,
                user_id,
            ))
            .await
            .ok_or(AuthError::SignupFailed)?;

        Ok(())
    }

    // 통합 회원가입
    //
    // Spring 쪽 통합 회원가입 로직을 러스트로 옮긴 형태라고 볼 수 있다.
    //
    // 흐름:
    // 1. 닉네임이 없으면 이름을 닉네임으로 사용
    // 2. user_type 확인
    // 3. STORE면 점주 회원가입 DTO로 변환해서 sign_up_store 호출
    // 4. 그 외는 일반 회원가입 DTO로 변환해서 sign_up_user 호출
    pub async fn sign_up(&self, dto: TotalSignUpRequestDto) -> Result<(), AuthError> {
        // 닉네임이 비어 있으면 이름을 기본 닉네임으로 사용한다.
        //
        // dto.user_nickname이 Option<String> 이라고 가정하면:
        // - Some(nickname)이면 그 값 사용
        // - None이면 dto.user_name.clone() 사용
        //
        // unwrap_or_else(...)는 기본값 계산이 필요한 경우 자주 쓴다.
        let nickname = dto.user_nickname.unwrap_or_else(|| dto.user_name.clone());

        // user_type을 보고 분기한다.
        //
        // as_deref():
        //   Option<String> -> Option<&str>처럼 비교하기 쉬운 형태로 바꿔준다.
        //
        // match dto.user_type.as_deref():
        //   - Some("STORE")이면 점주 회원가입
        //   - 그 외는 일반 회원가입
        match dto.user_type.as_deref() {
            Some("STORE") => {
                // STORE 회원가입이라면 점포 정보가 반드시 필요하다.
                //
                // dto.store_name / category / latitude / longitude 가 Option일 수 있으므로
                // ok_or(...)? 로 값이 없으면 즉시 실패시킨다.
                let store_name = dto.store_name.ok_or(AuthError::SignupFailed)?;
                let store_category = dto.store_category.ok_or(AuthError::SignupFailed)?;
                let store_latitude = dto.store_latitude.ok_or(AuthError::SignupFailed)?;
                let store_longitude = dto.store_longitude.ok_or(AuthError::SignupFailed)?;

                // 통합 DTO를 점주 회원가입 DTO로 재조립해서
                // sign_up_store(...)를 재사용한다.
                //
                // 즉 중복 로직을 다시 쓰지 않고 기존 메서드에 위임하는 구조다.
                self.sign_up_store(StoreSignUpRequestDto {
                    user_email: dto.user_email,
                    user_name: dto.user_name,
                    user_password: dto.user_password,
                    user_nickname: nickname,
                    user_number: dto.user_number,
                    store_name,
                    store_category,
                    store_latitude,
                    store_longitude,
                })
                .await
            }
            _ => {
                // STORE가 아니면 일반 회원가입으로 처리한다.
                //
                // 여기서 _ 는 "그 외 모든 경우"를 뜻한다.
                // 즉:
                // - Some("USER")
                // - Some("PENDING")
                // - None
                // 등도 전부 여기로 들어온다.
                self.sign_up_user(UserSignUpRequestDto {
                    user_email: dto.user_email,
                    user_name: dto.user_name,
                    user_password: dto.user_password,
                    user_nickname: nickname,
                    user_number: dto.user_number,
                })
                .await
            }
        }
    }

    // 로그인
    //
    // 전체 흐름:
    // 1. 이메일로 유저 조회
    // 2. 비밀번호 bcrypt 검증
    // 3. 탈퇴 계정 여부 확인
    // 4. user_id 확보
    // 5. access token / refresh token 생성
    // 6. USER_INFO 조회
    // 7. refresh token을 DB에 저장
    // 8. 로그인 응답 DTO 반환
    pub async fn login(&self, dto: LoginRequestDto) -> Result<LoginResponseDto, AuthError> {
        // 1) 이메일로 유저 조회
        //
        // 없으면 EmailNotFound
        let user = self
            .user_repository
            .find_by_user_email(&dto.user_email)
            .await
            .ok_or(AuthError::EmailNotFound)?;

        // 2) bcrypt 비밀번호 검증
        //
        // verify(입력 평문, DB 해시)
        //
        // unwrap_or(false):
        //   검증 중 에러가 나면 그냥 false 취급
        let password_match = verify(&dto.user_password, &user.user_password).unwrap_or(false);

        // 3) 비밀번호 불일치면 즉시 실패
        if !password_match {
            return Err(AuthError::PasswordNotMatch);
        }

        // 4) 탈퇴/비활성 계정 차단
        //
        // user_is_active == "N" 이면 비활성화된 계정으로 보는 구조다.
        if user.user_is_active == "N" {
            return Err(AuthError::AccountWithdraw);
        }

        // 5) user_id 확보
        let user_id = user.user_id.ok_or(AuthError::UserNotFound)?;

        // 6) access token 생성
        //
        // generate_token(user_id, email, user_type)
        //
        // 이메일이 Option<String>일 수 있으므로:
        // - 있으면 &str로 꺼내고
        // - 없으면 빈 문자열 사용
        let access_token = self.jwt_util.generate_token(
            user_id,
            user.user_email.as_deref().unwrap_or(""),
            &user.user_type,
        );

        // 7) refresh token 생성
        let refresh_token = self.jwt_util.generate_refresh_token(user_id);

        // 8) USER_INFO 조회
        //
        // 로그인 응답에 level/point를 넣기 위해 읽어오는 것으로 보인다.
        let user_info = self
            .user_info_repository
            .find_by_id(user_id)
            .await
            .ok_or(AuthError::UserInfoNotFound)?;

        // 9) 새 refresh token을 DB에 저장
        //
        // 이후 refresh API에서
        // "요청 토큰 == DB 저장 토큰" 인지 비교하는 데 사용된다.
        self.user_repository
            .update_refresh_token(user_id, &refresh_token)
            .await;

        // 10) 로그인 응답 DTO 생성 후 반환
        Ok(LoginResponseDto {
            // access token
            token: access_token,

            // refresh token
            // 보통 이후 컨트롤러에서 쿠키로 내려줄 수도 있다.
            refresh_token,

            // 아직 역할 확정 전인지 여부
            is_new_user: user.user_type == "PENDING",

            // 현재 유저 역할
            user_type: user.user_type.clone(),

            // 유저 PK
            user_id,

            // 이름 / 닉네임 / 이메일
            user_name: user.user_name,
            user_nickname: user.user_nickname,
            user_email: user.user_email.unwrap_or_default(),

            // USER_INFO의 상태값
            user_info_level: user_info.user_info_level,
            user_info_point: user_info.user_info_point,
        })
    }

    // 로그아웃
    //
    // 핵심은 refresh token을 DB에서 제거해서
    // 더 이상 재발급(refresh)이 안 되게 만드는 것이다.
    pub async fn logout(&self, user_id: i64) -> Result<(), AuthError> {
        // 1) user_id가 실제 존재하는 유저인지 확인
        self.user_repository
            .find_by_id(user_id)
            .await
            .ok_or(AuthError::UserNotFound)?;

        // 2) DB에 저장된 refresh token 제거
        self.user_repository.clear_refresh_token(user_id).await;

        Ok(())
    }

    // 회원 탈퇴
    //
    // hard delete(행 삭제)가 아니라 soft delete(상태값 변경) 방식이다.
    //
    // 즉 DB row를 실제로 지우지 않고,
    // "탈퇴/비활성화 상태"로 바꾸는 방식으로 보인다.
    pub async fn withdraw(&self, user_id: i64) -> Result<(), AuthError> {
        // 1) 유저 존재 확인
        self.user_repository
            .find_by_id(user_id)
            .await
            .ok_or(AuthError::UserNotFound)?;

        // 2) 탈퇴 처리
        //    아마 user_is_active = "N" 같은 업데이트일 가능성이 크다.
        self.user_repository.update_withdraw(user_id).await;

        Ok(())
    }

    // user_type 변경
    //
    // 주로 소셜 로그인 직후 USER / STORE 역할을 확정할 때 사용하는 메서드다.
    pub async fn update_user_type(&self, user_id: i64, user_type: &str) -> Result<(), AuthError> {
        // 1) 허용된 값인지 검증
        //
        // USER / STORE / PENDING 외의 값은 거부
        if user_type != "USER" && user_type != "STORE" && user_type != "PENDING" {
            return Err(AuthError::InvalidUserType);
        }

        // 2) 유저 존재 확인
        self.user_repository
            .find_by_id(user_id)
            .await
            .ok_or(AuthError::UserNotFound)?;

        // 3) 실제 user_type 업데이트
        self.user_repository
            .update_user_type(user_id, user_type)
            .await;

        Ok(())
    }

    // 프로필 수정
    //
    // 단순 닉네임 수정만이 아니라,
    // user_type 변경에 따라 USER_INFO와 STORE까지 함께 맞추는 로직이다.
    //
    // 흐름:
    // 1. 유저 조회
    // 2. user_type 검증
    // 3. USER_ 프로필 수정
    // 4. USER_INFO 조회/생성 후 역할에 맞게 값 조정
    // 5. STORE 타입인데 점포가 없으면 STORE_ 생성
    // 6. user_type 변경 내용을 반영한 새 access token 생성
    // 7. 응답 DTO 반환
    pub async fn update_profile(
        &self,
        user_id: i64,
        dto: UserProfileUpdateDto,
    ) -> Result<UserProfileUpdateResponseDto, AuthError> {
        // 1) 유저 조회
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await
            .ok_or(AuthError::UserNotFound)?;

        // 2) 허용된 user_type인지 검사
        if dto.user_type != "USER" && dto.user_type != "STORE" {
            return Err(AuthError::InvalidUserType);
        }

        // 3) USER_ 테이블의 닉네임 / user_type 업데이트
        self.user_repository
            .update_profile(user_id, &dto.user_nickname, &dto.user_type)
            .await;

        // 4) USER_INFO를 읽어서 역할에 맞게 보정한다.
        let user_info = match self.user_info_repository.find_by_id(user_id).await {
            // USER_INFO가 이미 있으면 기존 값을 가져와 일부 필드만 재조정
            Some(mut existing) => {
                if dto.user_type == "STORE" {
                    // 점주라면:
                    // - 미션 수행 횟수 0
                    // - 미션 생성 횟수 3
                    existing.user_info_mission_do = 0;
                    existing.user_info_mission_make = 3;
                } else {
                    // 일반 유저라면:
                    // - 미션 수행 횟수 10
                    // - 미션 생성 횟수 0
                    existing.user_info_mission_do = 10;
                    existing.user_info_mission_make = 0;
                }
                existing
            }

            // USER_INFO가 없고 STORE라면 점주용 기본 USER_INFO 새 생성
            None if dto.user_type == "STORE" => UserInfo::for_store_user(user_id),

            // USER_INFO가 없고 그 외면 일반 유저용 기본 USER_INFO 새 생성
            None => UserInfo::for_normal_user(user_id),
        };

        // 5) USER_INFO 저장
        //
        // insert/upsert/update 성격일 수 있다.
        self.user_info_repository
            .save(user_info)
            .await
            .ok_or(AuthError::SignupFailed)?;

        // 6) STORE 타입인데 실제 점포 정보가 아직 없으면 STORE_ 생성
        //
        // 예:
        // 소셜 로그인 후 처음 역할을 STORE로 확정하는 시점
        if dto.user_type == "STORE"
            && self
                .store_repository
                .find_by_user_id(user_id)
                .await
                .is_none()
        {
            // STORE 생성에 필요한 값이 없으면 실패
            let store_name = dto.store_name.ok_or(AuthError::SignupFailed)?;
            let store_category = dto.store_category.ok_or(AuthError::SignupFailed)?;
            let store_latitude = dto.store_latitude.ok_or(AuthError::SignupFailed)?;
            let store_longitude = dto.store_longitude.ok_or(AuthError::SignupFailed)?;

            // STORE_ 저장
            self.store_repository
                .save(&Store::new(
                    store_name,
                    store_category,
                    store_latitude,
                    store_longitude,
                    user_id,
                ))
                .await
                .ok_or(AuthError::SignupFailed)?;
        }

        // 7) user_type이 바뀌면 JWT 안의 클레임도 달라질 수 있으므로
        //    새 access token을 발급한다.
        let new_access_token = self.jwt_util.generate_token(
            user_id,
            user.user_email.as_deref().unwrap_or(""),
            &dto.user_type,
        );

        // 8) 최종 응답 반환
        Ok(UserProfileUpdateResponseDto {
            user_id,
            access_token: new_access_token,
        })
    }
}

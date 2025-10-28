use axum::{
    // 移除 Query，因为我们不再需要 locale 参数
    // extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
};
// 明确导入我们要用的组件
use fake::faker::address::raw::{CityName, PostCode, StreetName};
use fake::faker::company::raw::*;
use fake::faker::internet::raw::*;
use fake::faker::name::raw::*;
use fake::faker::phone_number::raw::*;
use fake::locales::*; // 只需 EN
use fake::Fake; // 导入 Fake trait
use serde::Serialize;
// 移除 Deserialize，因为不再需要 FakeDataParams
// use serde::Deserialize;

/* 移除 FakeDataParams 结构体
#[derive(Deserialize)]
pub struct FakeDataParams {
    locale: Option<String>,
}
*/

// 定义返回的数据结构 (不变)
#[derive(Serialize, Debug)]
pub struct VirtualIdentity {
    locale: String,
    name: String,
    address: String,
    phone: String,
    email: String,
    company: String,
    username: String,
}

// 辅助函数 (简化为只生成 EN)
fn create_us_identity() -> VirtualIdentity {
    VirtualIdentity {
        locale: "en_US".to_string(),
        name: Name(EN).fake(),
        address: format!(
            "{}, {}, {}",
            StreetName(EN).fake::<String>(),
            CityName(EN).fake::<String>(),
            PostCode(EN).fake::<String>()
        ),
        phone: PhoneNumber(EN).fake(),
        email: SafeEmail(EN).fake(),
        company: CompanyName(EN).fake(),
        username: Username(EN).fake(),
    }
}

// Axum 路由处理函数 (简化，不再接收 Query 参数)
pub async fn handle_get_fake_identity() -> impl IntoResponse {
    // 始终调用生成美国身份的函数
    let identity = create_us_identity();
    (StatusCode::OK, Json(identity))
}
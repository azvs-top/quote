/// 存储对象上传载荷。
///
/// - `filename`: 可选原始文件名，用于保留后缀或排障定位；不参与对象 key 的最终定义。
/// - `bytes`: 文件二进制内容，不能为空时由调用方在 service 层做业务校验。
#[derive(Debug, Clone)]
pub struct StoragePayload {
    pub filename: Option<String>,
    pub bytes: Vec<u8>,
}

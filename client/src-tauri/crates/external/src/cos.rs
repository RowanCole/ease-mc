const COS_REGION: &str = "ap-beijing";
const COS_BUCKET: &str = "ease-mc-1257344929";
const COS_SECRET_ID: &str = "AKIDa1X5X000000000000000000000000000";
const COS_SECRET_KEY: &str = "00000000000000000000000000000000";


pub async fn get_object(key:String) {
    let url = format!("https://{}.cos.{}.myqcloud.com/{}", COS_BUCKET, COS_REGION, key);
    let header = {
        "Authorization": "Bear ".to_string() + &COS_SECRET_KEY
    };
}















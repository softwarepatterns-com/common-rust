pub fn get_config_from_env(prefix: &str) -> (String, String, String) {
  dotenvy::dotenv().unwrap();

  let access_key = std::env::var(format!("{}_AWS_ACCESS_KEY_ID", prefix)).unwrap();
  let secret_key = std::env::var(format!("{}_AWS_SECRET_ACCESS_KEY", prefix)).unwrap();
  let region = std::env::var(format!("{}_AWS_REGION", prefix)).unwrap();

  (access_key, secret_key, region)
}

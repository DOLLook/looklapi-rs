use rudi::SingleOwner;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
struct Common {
    profile: String,
    server: Server,
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub name: String,
    pub port: i32,
}

#[derive(Clone, Deserialize)]
pub struct Mysql {
    pub uri: String,
}

#[derive(Clone, Deserialize)]
pub struct Mssql {
    pub uri: String,
}

#[derive(Clone, Deserialize)]
pub struct Mongodb {
    pub uri: String,
}

#[derive(Clone, Deserialize)]
pub struct Redis {
    pub host: String,
    pub port: i32,
    pub password: String,
    /// 超时时间(毫秒)
    pub timeout: i32,
}

#[derive(Clone, Deserialize)]
pub struct RabbitMQ {
    pub address: String,
}

#[derive(Clone, Deserialize)]
pub struct Consul {
    pub host: String,
    pub port: i32,
    pub secure: bool,
    pub health_check: String,
    /// 健康检查间隔(秒)
    pub health_check_interval: i32,
    /// 健康检查超时时间(秒)
    pub health_check_timeout: i32,
    /// 服务注册超时时间(秒)
    pub deregister_critical_service_after: i32, // 秒
}

#[derive(Clone, Deserialize)]
pub struct Logger {
    pub default: String,
}

#[derive(Clone, Deserialize)]
pub struct Dev {
    pub developer: String,
}

#[derive(Clone, Deserialize)]
struct Env {
    mysql: Option<Mysql>,
    mssql: Option<Mssql>,
    mongodb: Option<Mongodb>,
    redis: Option<Redis>,
    rabbitmq: Option<RabbitMQ>,
    consul: Option<Consul>,
    logger: Logger,
    dev: Option<Dev>,
}

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub profile: String,
    pub server: Server,
    pub mysql: Option<Mysql>,
    pub mssql: Option<Mssql>,
    pub mongodb: Option<Mongodb>,
    pub redis: Option<Redis>,
    pub rabbitmq: Option<RabbitMQ>,
    pub consul: Option<Consul>,
    pub logger: Logger,
    pub dev: Option<Dev>,
}

#[SingleOwner]
impl AppConfig {
    #[di]
    fn new() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let env = format!("{}/config/application.toml", manifest_dir);

        let config = config::Config::builder()
            .add_source(config::File::with_name(env.as_str()))
            .build()
            .unwrap();

        let common = config.try_deserialize::<Common>().unwrap();

        let env = if common.profile.eq("dev") {
            format!("{}/config/application-dev.toml", manifest_dir)
        } else if common.profile.eq("prod") {
            format!("{}/config/application-prod.toml", manifest_dir)
        } else {
            panic!("invalid profile {}", common.profile)
        };

        let cfg = config::Config::builder()
            .add_source(config::File::with_name(env.as_str()))
            .build()
            .unwrap();

        let env = cfg.try_deserialize::<Env>().unwrap();
        return Self {
            profile: common.profile,
            server: common.server,
            mysql: env.mysql,
            mssql: env.mssql,
            mongodb: env.mongodb,
            redis: env.redis,
            rabbitmq: env.rabbitmq,
            consul: env.consul,
            logger: env.logger,
            dev: env.dev,
        };
    }
}

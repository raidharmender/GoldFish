#[derive(Debug, Clone)]
pub enum ActorType {
  Customer,
  AdminApi,
  Institution,
  StormApi,
  Internal,
  Vendor,
}

#[derive(Debug, Clone)]
pub struct Actor {
  pub actor_type: ActorType,
  pub subject: String,
}


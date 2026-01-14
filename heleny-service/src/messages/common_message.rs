use heleny_proto::Resource;

#[derive(Debug, Clone)]
pub enum CommonMessage {
    Stop,
    Resource(Resource),
}

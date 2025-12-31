use heleny_proto::resource::Resource;

#[derive(Debug, Clone)]
pub enum CommonMessage {
    Stop,
    Resource(Resource),
}

use heleny_proto::resource::Resource;

#[derive(Debug)]
pub enum CommonMessage {
    Stop,
    Resource(Resource),
}

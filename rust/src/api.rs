#![allow(dead_code)]

use std::path::PathBuf;
use anyhow::Result;
use flutter_rust_bridge::ZeroCopyBuffer;
use protobuf::descriptor::{MethodDescriptorProto, ServiceDescriptorProto};
use protobuf::reflect::{FieldDescriptor, FileDescriptor, MessageDescriptor};

#[derive(Debug, Clone, Default)]
pub struct Proto {
    name: String,
    package: String,
    services: Vec<Service>,
    messages: Vec<Message>,
}

impl Proto {
    fn from_file(path: &str) -> Result<Self> {
        let mut proto = Proto::default();
        let file_descriptor = protobuf_parse::Parser::new()
            .protoc()
            .include(PathBuf::from(path).parent().unwrap())
            .input(path)
            .parse_and_typecheck()
            .map(|f| {
               FileDescriptor::new_dynamic(f.file_descriptors[0].clone(), vec![])
            })??;
        let file_descriptor_proto = file_descriptor.proto();
        proto.name = file_descriptor_proto.name().to_owned();
        proto.package = file_descriptor_proto.package().to_owned();
        proto.services = file_descriptor_proto.service.clone().into_iter().map(|s| Service::from_descriptor_proto(s)).collect();
        proto.messages = file_descriptor.messages().map(|m| Message::from_descriptor_proto(m)).collect();

        Ok(proto)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    name: String,
    fields: Vec<Field>,
}

impl Message {
    fn from_descriptor_proto(message_descriptor: MessageDescriptor) -> Self {
        let mut message = Message::default();
        message.name = message_descriptor.name().to_owned();
        message.fields = message_descriptor.fields().map(|f| Field::from_descriptor(f)).collect();

        message
    }
}

#[derive(Debug, Clone, Default)]
pub struct Field {
    name: String,
    field_type: FieldKind,
    optional: bool,
    repeated: bool,
}

impl Field {
    fn from_descriptor(field_descriptor: FieldDescriptor) -> Self {
        let mut field = Field::default();

        field.name = field_descriptor.name().to_owned();
        // FIXME: add type
        field.optional = field_descriptor.is_singular();
        field.repeated = field_descriptor.is_repeated();

        field
    }
}

#[derive(Debug, Clone)]
pub enum FieldKind {
    Unknown = 0,
    Double,
    Float,
    Int64,
    Uint64,
    Int32,
    Fixed64,
    Fixed32,
    Bool,
    String,
    Group,
    Message,
    Bytes,
    Uint32,
    Enum,
    Sfixed32,
    Sfixed64,
    Sint32,
    Sint64,
}

impl Default for FieldKind {
    fn default() -> Self {
        FieldKind::Unknown
    }
}

#[derive(Debug, Clone, Default)]
pub struct Service {
    name: String,
    methods: Vec<Method>,
}

impl Service {
    fn from_descriptor_proto(descriptor_proto: ServiceDescriptorProto) -> Self {
        let mut service = Service::default();
        service.name = descriptor_proto.name().to_owned();
        service.methods = descriptor_proto.method.into_iter().map(|m| Method::from_descriptor_proto(m)).collect();

        service
    }
}

#[derive(Debug, Clone, Default)]
pub struct Method {
    name: String,
    kind: MethodKind,
    input_type: String,
}

impl Method {
    fn from_descriptor_proto(method_descriptor_proto: MethodDescriptorProto) -> Self {
        let mut method = Method::default();

        method.name = method_descriptor_proto.name().to_owned();

        let client_streaming = method_descriptor_proto.has_client_streaming() as u8;
        let server_streaming = (method_descriptor_proto.has_server_streaming() as u8) << 1;
        match client_streaming & server_streaming {
            0b00 => method.kind = MethodKind::Unary,
            0b10 => method.kind = MethodKind::ClientStreaming,
            0b01 => method.kind = MethodKind::ServerStreaming,
            0b11 => method.kind = MethodKind::BidirectionalStreaming,
            _ => unreachable!()
        }

        method.input_type = method_descriptor_proto.input_type().to_owned();

        method
    }
}

#[derive(Debug, Clone)]
pub enum MethodKind {
    Unknown,
    Unary,
    ClientStreaming,
    ServerStreaming,
    BidirectionalStreaming,
}

impl Default for MethodKind {
    fn default() -> Self {
        MethodKind::Unknown
    }
}

// TODO: use a stream instead
pub fn load_proto_from_files(paths: Vec<String>) -> Result<ZeroCopyBuffer<Vec<Proto>>> {
    Ok(ZeroCopyBuffer(paths.into_iter().map(|p| Proto::from_file(&p).unwrap()).collect()))
}

#[test]
fn load_proto() {
    let proto = Proto::from_file(r"D:\Developer\Projects\GraduationProject\backend\service\proto\auth.proto").unwrap();
    println!("{:#?}", proto);
}

use std::{iter, vec};

use ftp::transfer_service_client::TransferServiceClient;
use ftp::{Chunk,Metadata,FileInfo,MessageInfo,TransferResponse,};
use tonic::{client,Request};
use futures_util::stream::iter;
pub mod ftp{
    tonic::include_proto!("ftp");
}

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error>>{
    let mut client=TransferServiceClient::connect("http://127.0.0.1:5051").await?;
    //for simple text msg
    let transfer_id="1".to_string();
    let sender_bank_id="banksender".to_string();
    let receiver_bank_id="bank_reciever".to_string();
    let msg="hello from client".to_string();
    let msg_metadata=Metadata{
        transfer_id,
        sender_bank_id,
        receiver_bank_id,
        payload_type:Some(ftp::metadata::PayloadType::MessageInfo(MessageInfo{
            length: msg.len() as u64,
        })),
    };
    let msg_chunk=Chunk{
        data:Some(ftp::chunk::Data::Content(msg.as_bytes().to_vec())),
    };
    let metadata_chunk=Chunk{
        data:Some(ftp::chunk::Data::Metadata(msg_metadata)),
    };
    let chunks=vec![metadata_chunk,msg_chunk];
    let stream=iter(chunks);
    let request=Request::new(stream);
    let response=client.transfer(request).await?;
    let transfer_response=response.into_inner();
    println!("Response {:?}",transfer_response);
    Ok(())
}
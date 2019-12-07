use rakrs_io::{Little, Triad};
use rakrs_protocol::online::inner::{
    InnerPacket, InnerPacketReliability as Reliability, Ordered, Reliable, Sequenced, Split,
};
use rakrs_protocol::online::Datagram;

const CHANNEL_COUNT: usize = 32;

#[derive(Default)]
pub struct SendQueue {
    mtu_size: usize,
    queue: Option<Vec<InnerPacket>>,
    est_size: usize,
    next_seq_number: u32,
    send_ordered_indices: [u32; CHANNEL_COUNT],
    send_sequenced_indices: [u32; CHANNEL_COUNT],
    message_index: u32,
    split_id: u16,
}

pub enum OrderType {
    Nil,
    Ordered { order_channel: u8 },
    Sequenced { order_channel: u8 },
}

impl SendQueue {
    pub fn push(&mut self, buffer: Vec<u8>, reliable: bool, order_type: OrderType, receipt: bool) {
        // TODO investigate the feasibility of passing in a lazy enum{CanIo, Vec<u8>} so that

        let reliable = if reliable {
            Some(Reliable {
                message_index: Default::default(),
            })
        } else {
            None
        };

        enum FatOrderType {
            Nil,
            Ordered(Ordered),
            Sequenced(Sequenced),
        }

        let fat = match order_type {
            OrderType::Nil => FatOrderType::Nil,
            OrderType::Ordered { order_channel } => FatOrderType::Ordered(Ordered {
                order_index: {
                    let r = &mut self.send_ordered_indices[order_channel as usize];
                    let ret = *r;
                    *r += 1;
                    Little::from(Triad::from(ret))
                },
                order_channel,
            }),
            OrderType::Sequenced { order_channel } => FatOrderType::Sequenced(Sequenced {
                sequence_index: {
                    let r = &mut self.send_sequenced_indices[order_channel as usize];
                    let ret = *r;
                    *r += 1;
                    Little::from(Triad::from(ret))
                },
                ordered: Ordered {
                    order_index: Little::from(Triad::from(
                        self.send_ordered_indices[order_channel as usize],
                    )),
                    order_channel,
                },
            }),
        };

        let reliability = if let Some(reliable) = reliable {
            match fat {
                FatOrderType::Nil => {
                    if receipt {
                        Reliability::ReliableWithAckReceipt(reliable)
                    } else {
                        Reliability::Reliable(reliable)
                    }
                }
                FatOrderType::Ordered(ordered) => {
                    if receipt {
                        Reliability::ReliableOrderedWithAckReceipt(reliable, ordered)
                    } else {
                        Reliability::ReliableOrdered(reliable, ordered)
                    }
                }
                FatOrderType::Sequenced(sequenced) => {
                    if receipt {
                        panic!("Reliable Sequenced packet cannot have ACK receipt")
                    } else {
                        Reliability::ReliableSequenced(reliable, sequenced)
                    }
                }
            }
        } else {
            match fat {
                FatOrderType::Nil => {
                    if receipt {
                        Reliability::UnreliableWithAckReceipt
                    } else {
                        Reliability::Unreliable
                    }
                }
                FatOrderType::Ordered(_) => panic!("Unreliable packet cannot be ordered"),
                FatOrderType::Sequenced(sequenced) => {
                    if receipt {
                        panic!("Unreliable Sequenced packet cannot have ACK receipt")
                    } else {
                        Reliability::UnreliableSequenced(sequenced)
                    }
                }
            }
        };

        let new_reliability = move |queue: &mut Self| {
            let mut ret = reliability.clone();
            if let Some(reliable) = ret.reliable_mut() {
                reliable.message_index = Little::from(Triad::from(queue.message_index));
                queue.message_index += 1;
            }
            ret
        };

        // https://github.com/pmmp/RakLib/blob/497a8e669203d5f8d2f54d01c2c980b8fc290f75/src/server/Session.php#L376-L377
        let max_size = self.mtu_size - 60;

        if buffer.len() <= max_size {
            let packet = InnerPacket {
                reliability: new_reliability(self),
                split: None,
                buffer,
            };
            self.push_inner(packet);
        } else {
            // TODO Let's try to prevent allocating O(n/m) vecs and directly write to a Datagram

            let chunks = buffer.chunks(max_size);
            let split_count = chunks.len();

            let split_id = self.split_id;
            self.split_id = split_id.wrapping_add(1);

            for (split_index, chunk) in chunks.enumerate() {
                let packet = InnerPacket {
                    reliability: new_reliability(self),
                    split: Some(Split {
                        split_count: split_count as u32,
                        split_id,
                        split_index: split_index as u32,
                    }),
                    buffer: chunk.to_vec(),
                };
                self.push_inner(packet);
            }
        }
    }

    fn push_inner(&mut self, packet: InnerPacket) {
        let size = packet.size();
        self.flush_if_long(size);

        self.queue.as_mut().unwrap().push(packet);
        self.est_size += size;

        self.flush_if_long(0);
    }

    fn flush_if_long(&mut self, extra: usize) {
        if self.est_size + 4 + 20 + 8 + 8 + extra > self.mtu_size {
            self.flush();
        }
    }

    fn flush(&mut self) {
        let _datagram = Datagram {
            seq_number: {
                let r = self.next_seq_number;
                self.next_seq_number += 1;
                r.into()
            },
            packets: self.queue.replace(vec![]).unwrap(),
        };

        // TODO handle NACK resending

        unimplemented!("Dispatch datagram")
    }
}

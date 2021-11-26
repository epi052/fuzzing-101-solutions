from scapy.all import *

PCAP_OUT = "corpus/bootp-testcase.pcap"

# create a somewhat normal looking baseline packet, port 68 is bootp server
base = IP(dst="127.1.1.1") / UDP(dport=68)

# add BOOTP header
pkt = base / BOOTP(op=1)  # bootp opcode: BOOTREQUEST

pcap = PcapWriter(PCAP_OUT, sync=True)
pcap.write_header(pkt)  # pcap header, read by libpcap
pcap.write_packet(pkt)  # actual packet

# ./build/sbin/tcpdump -r bootp-testcase.pcap
# 17:16:49.737595 IP view-localhost.bootps > localhost.bootpc: BOOTP/DHCP, Request from 00:00:00:00:00:00 (oui Ethernet), length 236
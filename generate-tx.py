
# 참고 - 이 스크립트를 사용하려면 Jeff Garzik의 python-bitcoinrpc가 필요하다
# https://github.com/jgarzik/python-bitcoinrpc

import os
import sys;
import json;
from bitcoinrpc.authproxy import AuthServiceProxy;

# 이 값들을 설정하라
rpc_user = "bitcoinrpc";
rpc_pass = "A7Xr149i7F6GxkhDbxWDTbmXooz1UZGhhyUYvaajA13Z";
rpc_host = "localhost";
rpc_port = 8332;

donation_minimum = 0;
donation_per_input = 3000;
donation_address = "1ForFeesAndDonationsSpendHerdtWbWy";


# 윈도우 공용 애플리케이션 데이터 폴더를 찾는 참고 링크
try:
    from win32com.shell import shellcon, shell            
    config_file = shell.SHGetFolderPath(0, shellcon.CSIDL_APPDATA, 0, 0) + "/Bitcoin/bitcoin.conf"
except ImportError: # non-windows/win32com 환경을 위한 간단한 대체 처리
    config_file = os.path.expanduser("~") + "/.bitcoin/bitcoin.conf"

# 이 함수는 ryan-c의 도움을 받았다
def asp_from_config(filename):
    rpcport = '8332'
    rpcconn = '127.0.0.1'
    rpcuser = None
    rpcpass = None
    with open(filename, 'r') as f:
        for line in f:
            try:
              (key, val) = line.rstrip().replace(' ', '').split('=')
            except:
              (key, val) = ("", "");
            if key == 'rpcuser':
                rpcuser = val
            elif key == 'rpcpassword':
                rpcpass = val
            elif key == 'rpcport':
                rpcport = val
            elif key == 'rpcconnect':
                rpcconn = val
        f.close()
    if rpcuser is not None and rpcpass is not None:
        rpcurl = 'http://%s:%s@%s:%s' % (rpcuser, rpcpass, rpcconn, rpcport)
        print('RPC server: %s' % rpcurl)
        return AuthServiceProxy(rpcurl)


def to_satoshi(s):
  return int (100000000 * float (s));
def from_satoshi(s):
  return float (s) / 100000000;


if len(sys.argv) < 3:
  print ("Usage: %s <input size> <target output size in BTC>" % sys.argv[0]);
  exit (0);

#service = AuthServiceProxy ("http://%s:%s@%s:%d" % (rpc_user, rpc_pass, rpc_host, rpc_port));
service = asp_from_config (config_file);

balance = to_satoshi (service.getbalance());
unspent = service.listunspent();
target_in  = to_satoshi (sys.argv[1]);
target_out = to_satoshi (sys.argv[2]);

if balance < target_in:
  print ("Cannot spend %f; only have %f in wallet." % (from_satoshi (target_in), from_satoshi (balance)));
  exit (0);

if target_out > target_in:
  print ("Please have a smaller target output than input value.");
  exit (0);


# 입력 찾기
# TODO: 더 똑똑한 코인 선택 알고리즘 적용
# 현재는 abs(value - target output) 증가 순으로 코인을 정렬한 뒤 순서대로 선택한다
inputs = [];
donation = 0;
total_in = 0;

unspent.sort (key=lambda coin: abs(to_satoshi (coin['amount']) - target_in));

for coin in unspent:
  total_in += to_satoshi (coin['amount']);
  donation += donation_per_input;
  inputs.append (dict (txid = coin['txid'], vout = coin['vout']));
  if total_in > target_in:
    break;

if donation < donation_minimum:
  donation = donation_minimum;

# 출력 찾기
outputs = dict ();
outputs[donation_address] = from_satoshi (donation);
total_in -= donation;
while total_in > target_out:
  outputs[service.getnewaddress()] = from_satoshi (target_out);
  total_in -= target_out;
outputs[service.getnewaddress()] = from_satoshi (total_in);

# 트랜잭션 생성
print service.createrawtransaction (inputs, outputs);






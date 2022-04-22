echo "Liberland Validator Setup Script"
echo "Before we start, make sure that you can cloned the repository and executed cargo build --release"
sleep 3
echo "Generating Validator Keys"
sleep 2
bash generate_stored_keys.sh
echo "Key's have been generated!"
sleep 1
echo "Generating node keys"
subkey generate-node-key --file node-key-file
sleep 2
echo "You can now start your validator with the following command:"
echo '''
./target/release/substrate --chain liberland-menger.json \
	--validator --in-peers 256 \  
	--base-path /liberland_chain/ \
	--unsafe-ws-external --rpc-cors all --rpc-external --rpc-methods=Unsafe 
	--name validator_name_here  \
	--node-key node-key-file \ 
	--bootnodes /ip4/206.221.189.10/tcp/30333/p2p/12D3KooWRm651Kd5GmsLTHJbgX5chQS5npx9ttLgo46UsegCMoNM \
	--telemetry-url "ws://telemetry.laissez-faire.trade:8001/submit 1"


'''
echo "after you have started you validator and let it sync up to the main chain, type yes:"
read awns
if [[ $string != "yes" ]]; then
  echo "no yes? bye"
  exit 1
fi 
echo "uploading keys to validator"
bash ig.sh
cp ig.sh my_validator_keys.txt
echo "keys uploaded!"
sleep 2
echo "Navigate to polkadot.js and setup a nominator and a "
sleep 5
echo "Copy the following result into polkadot js > Staking > Account Actions, and click Session Key with the following result: "
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' http://localhost:9933
echo "keys rotated ok"
sleep 1
echo "Keys have been inserted into your validator, please restart your validator quickly after adding it as a validator using polkadot.js and remove the rpc flags"
echo "all done!"

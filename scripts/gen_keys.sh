#!/usr/bin/env bash
set -e

# if [ "$#" -ne 1 ]; then
# 	echo "Please provide the number of initial validators!"
# 	exit 1
# fi

# Copy paste your mnemonic here.
# SECRET="panda dose welcome ostrich brief pull lawn table arrest worth ranch faculty"

# generate_account_id() {
# 	subkey inspect ${2:-} ${3:-} "$SECRET//$1" | grep "Account ID" | awk '{ print $3 }'
# }
generate_secret() {
	# SECRET=subkey generate | grep "Secret phrase" | awk '{ print }' | awk '{printf "%s}'
	SECRET=`subkey generate | grep "Secret phrase" | awk '{printf $1=""; $2=""; print}'`
}

generate_address() {
	subkey inspect ${2:-} ${3:-} "$SECRET//$1" | grep "SS58 Address" | awk '{ print $3 }'
}

generate_public_key() {
	subkey inspect ${2:-} ${3:-} "$SECRET//$1" | grep "Account ID" | awk '{ print $3 }'
}

# generate_address_and_public_key() {
# 	ADDRESS=$(generate_address $1 $2 $3)
# 	PUBLIC_KEY=$(generate_public_key $1 $2 $3)

# 	printf "Address		: ${ADDRESS}"
# 	printf "Account ID	: ${PUBLIC_KEY#'0x'}"
# }

generate_address_and_account_id() {
	ADDRESS=$(generate_address $1 $2)
	PUBLIC_KEY=$(generate_public_key $1 $2)

	printf "Key Type	: $1\n"
	printf "Address		: ${ADDRESS}\n"
	printf "Account ID	: ${PUBLIC_KEY#'0x'}\n"
}

generate_secret 
printf "\nSecret 		:${SECRET}"

AUTHORITIES="\n\n"
# AUTHORITIES+="$(generate_secret)\n"
AUTHORITIES+="$(generate_address_and_account_id stash)\n\n"
AUTHORITIES+="$(generate_address_and_account_id controller)\n\n"
AUTHORITIES+="$(generate_address_and_account_id grandpa '--scheme ed25519')\n\n"
AUTHORITIES+="$(generate_address_and_account_id babe '--scheme sr25519')\n\n"
AUTHORITIES+="$(generate_address_and_account_id im_online '--scheme sr25519')\n\n"
AUTHORITIES+="$(generate_address_and_account_id authority_discovery '--scheme sr25519')\n\n"
# AUTHORITIES+="),\n"

printf "$AUTHORITIES"
# printf "$SECRET"

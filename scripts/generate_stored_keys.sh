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

copy_files() {
	touch ig.sh && cp ig.sh ig.sh.old
	cp insert_keys_gen.sh ig.sh 
}

generate_address_and_account_id() {
	ADDRESS=$(generate_address $1 $2)
	PUBLIC_KEY=$(generate_public_key $1 $2)
	ACCOUNT_PREFIX="ACCOUNT_$1"
	M="MNEMONIC_$1"
	SK="${SECRET}//$1"
	SD="s/$M/$SECRET/g"
	printf "SD:	 s/$M/${SECRET:2}/g \n"
	printf "AC:	 s/$ACCOUNT_PREFIX/0x$PUBLIC_KEY/g \n"
	sed -i "s/$M/${SECRET:2}/g" ig.sh
	sed -i "s/$ACCOUNT_PREFIX/${PUBLIC_KEY#'0x'}/g" ig.sh
	printf "Key Type	: $1\n"
	printf "Address		: ${ADDRESS}\n"
	printf "A Prefix		: ${ACCOUNT_PREFIX}\n"
	printf "Account ID	: ${PUBLIC_KEY#'0x'}\n"
	printf "M Prefix		: ${M}\n"
	printf "Secret		: ${SK} \n"
}

copy_files
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
printf "run ig.sh now"

#!/bin/bash
uri_decode() {
    local encoded_str="$1"
    printf -v decoded_str '%b' "${encoded_str//%/\\x}"
    echo "$decoded_str"
}
# Check if there is a client certificate
if [[ -z "${TLS_CLIENT_HASH}" ]]; then
	echo -ne "60 text/gemini\r\n"
else
	if [ "${QUERY_STRING}" = "emailrequest" ]
	then
	echo -ne "10 text/gemini Please fill in you e-mail address.Thank you!\r\n"
	else

		if [[ -z "${QUERY_STRING}" ]];
		then
			echo -ne "20 text/gemini\r\n"
			echo "# Subscription "
			echo "We have received your Client Certificate."
			echo "After confirmation of your e-mail adress you will be granted access to our protected pages"
			echo "=>subscribe.sh?emailrequest Please Fill in your e-mail address"
		else
			# Check for e-mail adress ,check if string contains an '@' and at least 1 dot and longer dan 5 chars
			if [[ ${#QUERY_STRING} -gt 5 && "$QUERY_STRING" == *%40*.* ]];then
				decoded_str=$(uri_decode "${QUERY_STRING}")
				echo "${decoded_str}::${TLS_CLIENT_HASH}" >> to_be_confirmed.keys;
				echo -ne "20 text/gemini\r\n"
				echo "# Subscription "
				echo "Thank you for your subscription."
				echo "After confirmation of your e-mail address you will be granted access to our protected pages"
				echo -ne "\r\n"
			# Check for e-mail adress ,check if string contains an '@' and at least 1 dot and longer dan 5 chars
			else 
				echo -ne "20 text/gemini\r\n"
				echo "# Subscription "
				echo "## Please provide a correct e-mail address"
				echo "After confirmation of your e-mail adress you will be granted access to our protected pages"
				echo "=>subscribe.sh?emailrequest Please Fill in your e-mail address"
				echo -ne "\r\n"
			fi
		fi
	fi
fi

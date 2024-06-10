#!/bin/bash
# Fetches any man page available on your system
if [ ${QUERY_STRING} =="" ]
then
echo -ne "10 input\r\n"
else 
     echo -ne "20 text/gemini\r\n"
     man ${QUERY_STRING}
fi


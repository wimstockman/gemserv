 #!/bin/bash
MEMINFO=$(cat /proc/meminfo|grep Mem)
CPUINFO=$(cat /proc/cpuinfo |egrep 'vendor|MHz|model name|cache size'
echo -ne "20 text/gemini\r\n"
cat <<EOGEMINI
Server Info
## Mem info
${MEMINFO}
## CPU info
${CPUINFO}
### Generated by: ${0}
EOGEMINI

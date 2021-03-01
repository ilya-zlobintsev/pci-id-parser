#!/bin/sh
pci-parser &&
patch < patch.diff &&
#echo $(date -u "+%y/%m/%d %H:%M") > date
echo $(date -u "+%s") > date &&
cp devices.json /srv/http/ &&
cp date /srv/http 

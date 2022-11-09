cd target/debug
cp libnss_cosiauthd.so libnss_cosiauthd.so.2
scp libnss_cosiauthd.so.2 tiamat:~
#install -m 0644 libnss_cosiauthd.so.2 /lib
#/sbin/ldconfig -n /lib /usr/lib


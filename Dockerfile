FROM intel/oneapi-basekit:devel-ubuntu22.04
COPY os /
COPY buffer_size_* /
COPY vtune_all.sh /

LABEL authors="hennickc"

ENTRYPOINT ["/vtune_all.sh"]
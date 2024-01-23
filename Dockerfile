FROM intel/oneapi-basekit:devel-ubuntu22.04
COPY ./vtune_all.sh /
COPY ./os /
COPY ./buffer_size_* /

LABEL authors="hennickc"

ENTRYPOINT ["/vtune_all.sh"]
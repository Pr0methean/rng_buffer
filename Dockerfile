FROM intel/oneapi-basekit:devel-ubuntu22.04
COPY ./vtune_all.sh /
COPY ./os /
COPY ./buffer_size_* /

USER root

LABEL authors="hennickc"

ENTRYPOINT ["/vtune_all.sh"]
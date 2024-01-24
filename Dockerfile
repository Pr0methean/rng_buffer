FROM intel/oneapi-basekit:devel-ubuntu22.04
COPY ./vtune_all.sh /
COPY ./os /
COPY ./buffer_size_* /

USER root

RUN curl --insecure "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
RUN unzip awscliv2.zip
RUN ./aws/install

LABEL authors="hennickc"

ENTRYPOINT ["/vtune_all.sh"]
FROM public.ecr.aws/x8v8d7g8/mars-base:latest
WORKDIR /app
RUN pip install -r requirements/test_extra_new.txt
RUN pip install -e .
CMD ["/bin/bash"]

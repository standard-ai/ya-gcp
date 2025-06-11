.PHONY: bash
bash:
	@docker-compose run --service-ports --rm app bash ${CMD_ARGS}

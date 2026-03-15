COMPONENTS := auth storage secretmanager pubsub tasks bigquery

.PHONY: all
all:
	@for dir in $(COMPONENTS); do echo "--- $$dir ---" && $(MAKE) -C $$dir build; done

.PHONY: clean
clean:
	@for dir in $(COMPONENTS); do echo "--- $$dir ---" && $(MAKE) -C $$dir clean; done

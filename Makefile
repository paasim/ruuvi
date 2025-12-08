.PHONY: check

check: .git/hooks/pre-commit
	. $<

.git/hooks/pre-commit:
	curl -o $@ https://gist.githubusercontent.com/paasim/317a1fd91a6236ca36d1c1c00c2a02d5/raw/315eb5b4e242684d64deb07a0c1597057af29f90/rust-pre-commit.sh
	echo "" >> $@
	chmod +x $@

.PHONY: html
	
tailwind:
	npx tailwindcss -i ./templates/input.css -o ./html/styles.css
tailwind-watch:
	npx tailwindcss -i ./templates/input.css -o ./html/styles.css --watch
html:
	cargo run -- -i '${HOME}/Sync/PARK/Area/Publish/**/*' -o `pwd`
html-watch:
	cargo watch -x "run -- -i '${HOME}/Sync/PARK/Area/Publish/**/*' -o `pwd`"
netlify:
	npx netlify deploy --dir=html
	# requires running `npx netlify login`
netlify-prod:
	npx netlify deploy --prod --dir=html
send: html netlify-prod

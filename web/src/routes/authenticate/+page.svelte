<h2>Autheticate with password</h2>
<div>
	<form
		on:submit|preventDefault={async () => {
			let resp = await fetch('/api/authenticate', {
				method: 'POST',
				headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
				body: new URLSearchParams({})
			});
			console.log(resp);
		}}
	>
		<label for="auth-user">User name:</label>
		<input type="text" name="user" id="auth-user" required /><br />
		<label for="auth-password">Password:</label>
		<input type="password" name="password" id="auth-password" required /><br />
		<input type="submit" />
	</form>
</div>

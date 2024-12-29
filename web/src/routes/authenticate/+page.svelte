<script lang="ts">
	import { page } from '$app/state';

	let user = $state('');
	let password = $state('');

	let sessionId = $state('');
	let sessionIdProvided = $state(false);
	$effect(() => {
		let sId = page.url.searchParams.get('session_id');
		if (sId == null) {
			sessionIdProvided = false;
			sessionId = '';
		} else {
			sessionIdProvided = true;
			sessionId = sId;
		}
	});

	let authResult = $state('');
</script>

<h2>Authenticate with password</h2>
<div>
	<form
		onsubmit={async (e) => {
			e.preventDefault();
			let resp = await fetch('/api/authenticate', {
				method: 'POST',
				headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
				body: new URLSearchParams({ session_id: sessionId, user: user, password: password })
			});
			console.log(resp);
			let respContent = await resp.json();
			console.log(respContent);
			if (respContent.error) {
				authResult = `Error: ${respContent.error}`;
			} else if (respContent.success) {
				authResult = `Error: ${respContent.success}`;
			}
		}}
	>
		<label for="auth-session-id">Session ID: </label>
		<input
			type="text"
			name="session_id"
			id="auth-session-id"
			disabled={sessionIdProvided}
			bind:value={sessionId}
		/><br />
		<label for="auth-user">User name:</label>
		<input type="text" name="user" id="auth-user" required bind:value={user} /><br />
		<label for="auth-password">Password:</label>
		<input type="password" name="password" id="auth-password" required bind:value={password} /><br
		/>
		<input type="submit" />
	</form>
	<p>{authResult}</p>
</div>

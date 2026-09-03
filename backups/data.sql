SET session_replication_role = replica;

--
-- PostgreSQL database dump
--

-- \restrict EowdIaddKETgsvymLIvj2TCBMZf9WCcQnDUQCXvCTP21X61IN4yOVEX55lIyBAa

-- Dumped from database version 17.6
-- Dumped by pg_dump version 17.6

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: audit_log_entries; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."audit_log_entries" ("instance_id", "id", "payload", "created_at", "ip_address") FROM stdin;
\.


--
-- Data for Name: custom_oauth_providers; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."custom_oauth_providers" ("id", "provider_type", "identifier", "name", "client_id", "client_secret", "acceptable_client_ids", "scopes", "pkce_enabled", "attribute_mapping", "authorization_params", "enabled", "email_optional", "issuer", "discovery_url", "skip_nonce_check", "cached_discovery", "discovery_cached_at", "authorization_url", "token_url", "userinfo_url", "jwks_uri", "created_at", "updated_at", "custom_claims_allowlist") FROM stdin;
\.


--
-- Data for Name: flow_state; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."flow_state" ("id", "user_id", "auth_code", "code_challenge_method", "code_challenge", "provider_type", "provider_access_token", "provider_refresh_token", "created_at", "updated_at", "authentication_method", "auth_code_issued_at", "invite_token", "referrer", "oauth_client_state_id", "linking_target_id", "email_optional") FROM stdin;
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."users" ("instance_id", "id", "aud", "role", "email", "encrypted_password", "email_confirmed_at", "invited_at", "confirmation_token", "confirmation_sent_at", "recovery_token", "recovery_sent_at", "email_change_token_new", "email_change", "email_change_sent_at", "last_sign_in_at", "raw_app_meta_data", "raw_user_meta_data", "is_super_admin", "created_at", "updated_at", "phone", "phone_confirmed_at", "phone_change", "phone_change_token", "phone_change_sent_at", "email_change_token_current", "email_change_confirm_status", "banned_until", "reauthentication_token", "reauthentication_sent_at", "is_sso_user", "deleted_at", "is_anonymous") FROM stdin;
\.


--
-- Data for Name: identities; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."identities" ("provider_id", "user_id", "identity_data", "provider", "last_sign_in_at", "created_at", "updated_at", "id") FROM stdin;
\.


--
-- Data for Name: instances; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."instances" ("id", "uuid", "raw_base_config", "created_at", "updated_at") FROM stdin;
\.


--
-- Data for Name: oauth_clients; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."oauth_clients" ("id", "client_secret_hash", "registration_type", "redirect_uris", "grant_types", "client_name", "client_uri", "logo_uri", "created_at", "updated_at", "deleted_at", "client_type", "token_endpoint_auth_method") FROM stdin;
\.


--
-- Data for Name: sessions; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."sessions" ("id", "user_id", "created_at", "updated_at", "factor_id", "aal", "not_after", "refreshed_at", "user_agent", "ip", "tag", "oauth_client_id", "refresh_token_hmac_key", "refresh_token_counter", "scopes") FROM stdin;
\.


--
-- Data for Name: mfa_amr_claims; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."mfa_amr_claims" ("session_id", "created_at", "updated_at", "authentication_method", "id") FROM stdin;
\.


--
-- Data for Name: mfa_factors; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."mfa_factors" ("id", "user_id", "friendly_name", "factor_type", "status", "created_at", "updated_at", "secret", "phone", "last_challenged_at", "web_authn_credential", "web_authn_aaguid", "last_webauthn_challenge_data") FROM stdin;
\.


--
-- Data for Name: mfa_challenges; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."mfa_challenges" ("id", "factor_id", "created_at", "verified_at", "ip_address", "otp_code", "web_authn_session_data") FROM stdin;
\.


--
-- Data for Name: oauth_authorizations; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."oauth_authorizations" ("id", "authorization_id", "client_id", "user_id", "redirect_uri", "scope", "state", "resource", "code_challenge", "code_challenge_method", "response_type", "status", "authorization_code", "created_at", "expires_at", "approved_at", "nonce") FROM stdin;
\.


--
-- Data for Name: oauth_client_states; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."oauth_client_states" ("id", "provider_type", "code_verifier", "created_at") FROM stdin;
\.


--
-- Data for Name: oauth_consents; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."oauth_consents" ("id", "user_id", "client_id", "scopes", "granted_at", "revoked_at") FROM stdin;
\.


--
-- Data for Name: one_time_tokens; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."one_time_tokens" ("id", "user_id", "token_type", "token_hash", "relates_to", "created_at", "updated_at") FROM stdin;
\.


--
-- Data for Name: refresh_tokens; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."refresh_tokens" ("instance_id", "id", "token", "user_id", "revoked", "created_at", "updated_at", "parent", "session_id") FROM stdin;
\.


--
-- Data for Name: sso_providers; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."sso_providers" ("id", "resource_id", "created_at", "updated_at", "disabled") FROM stdin;
\.


--
-- Data for Name: saml_providers; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."saml_providers" ("id", "sso_provider_id", "entity_id", "metadata_xml", "metadata_url", "attribute_mapping", "created_at", "updated_at", "name_id_format") FROM stdin;
\.


--
-- Data for Name: saml_relay_states; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."saml_relay_states" ("id", "sso_provider_id", "request_id", "for_email", "redirect_to", "created_at", "updated_at", "flow_state_id") FROM stdin;
\.


--
-- Data for Name: sso_domains; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."sso_domains" ("id", "sso_provider_id", "domain", "created_at", "updated_at") FROM stdin;
\.


--
-- Data for Name: webauthn_challenges; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."webauthn_challenges" ("id", "user_id", "challenge_type", "session_data", "created_at", "expires_at") FROM stdin;
\.


--
-- Data for Name: webauthn_credentials; Type: TABLE DATA; Schema: auth; Owner: supabase_auth_admin
--

COPY "auth"."webauthn_credentials" ("id", "user_id", "credential_id", "public_key", "attestation_type", "aaguid", "sign_count", "transports", "backup_eligible", "backed_up", "friendly_name", "created_at", "updated_at", "last_used_at") FROM stdin;
\.


--
-- Data for Name: access_tokens; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."access_tokens" ("token", "created_at", "reserved_email", "reserved_at", "redeemed_by", "redeemed_at", "revoked_at", "expires_at") FROM stdin;
dd131783-4cdf-4de8-b736-6265f9737790	1784807292	\N	\N	44c4948c-b76d-4523-8006-9581b036cea2	1784807340	\N	\N
b001be28-cf9a-44c6-9e9a-aea722413ab4	1784807376	\N	\N	c062e70d-7ef4-491c-8828-e1350f73ddc8	1784807406	\N	\N
0173b69d-fcbd-4605-8906-285efe7ba14d	1784811782	\N	\N	38e40fd4-f86d-4b52-8fa1-c8e8e4fc162d	1784811824	\N	\N
ad56dd39-086c-4c3b-a63c-5515fb1e073f	1784862525	\N	\N	4c012227-f0e6-41fd-832f-907a184b5857	1784862725	\N	\N
40cb13ba-9c78-4ff1-bc87-6de3f359415f	1784887362	\N	\N	0024e30d-71ed-4198-9dbf-57f3e95998c8	1784887367	\N	\N
6ac1155f-c06d-4a53-b0dd-28e21de06310	1784899648	\N	\N	fb09653a-4107-4adf-87a2-4ff2440677ca	1784899654	\N	\N
1e16de05-ae4c-44da-b55b-e254f9669108	1785132475	\N	\N	c0f45227-660a-4a76-b4da-98b19d781064	1785132482	\N	\N
968d402b-70d5-4434-9d9c-27ac7bfa979b	1785147343	\N	\N	195c1582-db2f-4055-aeeb-c54b5f6ccfa3	1785147356	\N	\N
e343fbab-ed1c-461d-a082-9f2763b04ccf	1785147626	\N	\N	e63e2216-b326-45d1-a390-45e220f6c861	1785147632	\N	\N
7bf4bd7f-a7a0-4041-a2be-76afe7230475	1785216669	\N	\N	58337c4b-72ba-4229-bc30-e9f90f6efc14	1785216674	\N	\N
cd46547a-ac82-4a06-b1da-7cbbb8af21a0	1785223327	\N	\N	d1206ad9-f01b-48e8-ac00-b7f4bcf4b63a	1785223332	\N	\N
d97ebf85-79fd-48b7-a8cb-de5aa27a53b2	1785311043	\N	\N	dfbb3ed5-8623-4f80-a728-c96de6eb77ea	1785311052	\N	\N
f73630ef-b056-4451-9d40-b390d6e46a56	1785312060	\N	\N	3ec1cee0-0c96-4345-8c5e-93f2c4e726e7	1785312069	\N	\N
3dfd4468-389b-42f8-954b-c6c896c1ddcb	1785312583	\N	\N	53e14aaf-6ef2-428f-a506-17309494c9fa	1785312587	\N	\N
d49b8b4d-7b6f-400e-aa46-0de7836d1a95	1785312889	\N	\N	bdf8af47-610b-4d69-b86f-29415b703d7d	1785312893	\N	\N
26c7e7bd-70da-40db-8a81-2d905bedacfe	1785313832	\N	\N	48339372-3066-4168-9658-0dd812512c5b	1785313835	\N	\N
2506410e-80aa-4e17-8c71-3a30ac74d090	1785314714	\N	\N	21665ba9-6be2-46db-8bd0-10df1d7d7822	1785314718	\N	\N
cc142f25-2963-46b9-8ec7-3fb2cacb3ab8	1785314872	\N	\N	3353fad4-28dc-4432-a090-2a515bf46f8c	1785314876	\N	\N
41476deb-94d9-4551-90a3-e9eaab8a2b24	1785378480	\N	\N	\N	\N	\N	\N
afe131cb-4c32-485e-be49-92cc70627956	1785379300	\N	\N	4a1fee23-718f-4677-8fd5-6d8f78463411	1785379303	\N	\N
8bb4a887-a815-466b-ade3-326ca4fc3755	1785383054	\N	\N	508ad9fd-c6d7-4ff6-b09c-ccf0cde4d6d8	1785383062	\N	\N
139e4feb-a52d-4055-8bac-fb77e10571a2	1785384618	\N	\N	81ce8e27-1745-4ed9-abe4-65bbc8ba8012	1785384622	\N	\N
973ee916-e721-41c9-ac21-468e6905c808	1785385017	\N	\N	787c8487-9a54-411e-abc2-174cf4fa7c09	1785385020	\N	\N
327a821b-5fc8-40c3-98f3-6111ccbfdfad	1785385599	\N	\N	0bbd5e2b-9966-4c99-a153-17e93ac8b8ae	1785385604	\N	\N
b90f586b-b35b-4ca3-8539-d301e1af9da8	1785385968	\N	\N	9a81fb0c-f487-4b12-9102-8aa6ea22ecff	1785385980	\N	\N
56478a3e-d96d-4589-891c-38f73fff26e9	1785386167	\N	\N	194a87ae-a361-410b-920d-1f5a2f9eb5ba	1785386171	\N	\N
5dd92ac6-b362-4680-9112-803249f1ef2c	1785386986	\N	\N	604af0a9-ea54-44df-9af0-4c8e87705753	1785386991	\N	\N
9e49c2b9-19c7-4d40-89b4-0a28bc409369	1785387736	\N	\N	36e9e804-ed7a-45c6-bcf1-87daa36e74ca	1785387739	\N	\N
458fd280-35d4-48d5-9b9a-955b324911b9	1785389149	\N	\N	8fcbfd05-41bc-4258-90d6-9ac255939824	1785389154	\N	\N
ef74b6cc-5837-4210-b17e-1e4ce356fa94	1785389593	\N	\N	a36ddd38-40c7-4c1a-8da7-cf490cbd36b0	1785389598	\N	\N
ac599574-9a4a-46d0-8ced-5a623ff613ab	1785400574	\N	\N	904c9ced-a24b-4887-8d6e-01feda354383	1785400578	\N	\N
ea56922b-81b2-4b61-bde6-8818a923b77f	1785650218	\N	\N	\N	\N	\N	\N
097a17bc-3119-42d0-8236-31c80f73e45c	1785651952	\N	\N	02041dc4-d46a-4c26-b973-f3880ce58b8e	1785651958	\N	\N
489de5ba-5f9d-4070-887e-6917afc4a1fd	1785652265	\N	\N	5bcc9fcc-0cca-44da-834a-20a307511776	1785652268	\N	\N
ebcf209b-a0d4-4714-b494-f8c97d9b18a1	1785653830	\N	\N	2b71c15a-8dbf-4d49-a887-7a58c35612df	1785653836	\N	\N
17cb0fa9-e9b1-4687-a1c2-40720adccc35	1785658933	\N	\N	2ae71be4-efe6-498d-a05f-4b7fd7a44882	1785658937	\N	\N
cc9c5d76-72e5-4cce-83e2-4fc867862f1f	1786252019	\N	\N	4f7d4ac8-9d67-40d8-9f44-23bdcc0ad903	1786252086	\N	\N
cd8a6b48-2940-445a-bc1a-a0d5571818c5	1786545308	\N	\N	72ae543a-2c39-4259-a4e6-9c0068002b56	1786545312	\N	\N
f6fadef1-28ec-41ac-b3bf-5fc8aa870482	1787634617	\N	\N	d5079571-1934-44eb-90f3-b448b560986a	1787634622	\N	\N
fece06ba-a983-44ad-a2dc-5482390b146a	1787722776	\N	\N	7f258a62-7e0e-44a4-8cdf-0641aee2459c	1787722794	\N	\N
d1e32b59-cb6f-4a8f-9e63-19601112b075	1787722828	\N	\N	e84db191-2a42-4705-a97d-f78035dcd66b	1787722835	\N	\N
3f3f2bd8-98aa-41bf-b200-99466fe09c39	1787794686	\N	\N	2eb4d454-1a6a-47e1-8cc8-1283ca715128	1787794713	\N	\N
d4c19932-b3b8-4ee9-8d27-e290bd17e6a8	1787802384	\N	\N	67abc61a-b515-4304-827f-2159114cc644	1787802405	\N	\N
\.


--
-- Data for Name: company_settings; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."company_settings" ("id", "company_name", "office_address", "currency", "timezone", "updated_at", "agent_commission_split_months") FROM stdin;
1	Thessalieh Property Consultancy		PHP	Asia/Manila	1783394482	15
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."users" ("id", "picture", "username", "email", "password_hash", "lastname", "firstname", "middlename", "role", "created_at", "updated_at", "failed_login_attempts", "lockout_until", "phone") FROM stdin;
2ae71be4-efe6-498d-a05f-4b7fd7a44882	\N	ofelia	ofelia@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$Pf9D76S3Wz5G0tf5Ef2gOQ$BFARQkP77nlTVbAwJyc5IA3Epg6LZmfe/NkPqJH5g3Q	\N	\N	\N	User	1785658937	1785658937	0	0	\N
a36ddd38-40c7-4c1a-8da7-cf490cbd36b0	\N	robert	robert@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$92hzQD6k/v5hU4dMcw7NBw$9xSe5vnH+yDsYd7iCkmeUslTBdBJj2mo7hlWKQXIkgw	Doldolia	Robert	\N	Agent	1785389598	1785389639	0	0	\N
dfbb3ed5-8623-4f80-a728-c96de6eb77ea	\N	marimon	marimon@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$UGGOMjyJiBERc8lehyfPgw$u5jG/Ul2bGtNFuFC5wMYi9PMajjaog2/7cGIFX92mA8	\N	\N	\N	User	1785311052	1785311052	0	0	\N
2b71c15a-8dbf-4d49-a887-7a58c35612df	\N	marces	marces@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$1nMFbqj8hoZIXN2dIB62AQ$u88osadX4D4UkNKY3Bp113qBDNI4zWRruuZ0izE+LdM	\N	\N	\N	User	1785653836	1785653836	0	0	\N
48339372-3066-4168-9658-0dd812512c5b	\N	pastor	pastor@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$PvbLF26xdmbmwt1P1tftUA$V8bZiB11nwHZ9XviBU9BA88/ouJKQYdfUnOS2GmRilE	\N	\N	\N	User	1785313835	1785313835	0	0	\N
194a87ae-a361-410b-920d-1f5a2f9eb5ba	\N	calzada	calzada@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$Wtcp1mf8qvdEG34qBdup2A$NKJfkYjf8zOV8sVMrElRRwAdNh0gbCM/WZ0kmhp2YCU	Calzada	Monaliza	\N	Agent	1785386171	1785386225	0	0	\N
3353fad4-28dc-4432-a090-2a515bf46f8c	\N	escalera	escalera@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$X2zxRobThHoFYER120lCQA$OI0dnqDSb8RQ1hXK7jk5C3IfxvEwVr8em6llxo80fvc	\N	\N	\N	User	1785314876	1785314876	0	0	\N
c062e70d-7ef4-491c-8828-e1350f73ddc8	\N	rosa	rosa@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$VO0yjPLuuQPhLoSptMEN8w$pRHRYOKM/d/Ve7LHsdlrREGlOFFAV5J1KDypp4PmeUY	Balani	Rosa		Titling Officer	1784807406	1784851132	0	0	\N
02041dc4-d46a-4c26-b973-f3880ce58b8e	\N	dongiapon	dongiapon@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$7JReGQCs5oZXerIWJKurbw$H0fUTJ7bGWQI70IzxKtJJTSaB7DLAYlXAOoLMQwulKs	\N	\N	\N	User	1785651958	1785651958	0	0	\N
0bbd5e2b-9966-4c99-a153-17e93ac8b8ae	\N	lumingkit	lumingkit@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$5P65HnSCcscSfcZ4LryqPg$wsZYwICtjULifdnpIOa18VKhjBIHvFfeR8Vb63xzlI8	\N	\N	\N	User	1785385604	1785385604	0	0	\N
21665ba9-6be2-46db-8bd0-10df1d7d7822	\N	jean	jean@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$c+B5OOp3+HtNT+skFChtyQ$8yBZZsXqHwHmB9nAwaUH3ZI5ETJUjtV4BAa1gtrPgZ0	Doldolia	Jennifer	\N	Agent	1785314718	1786006702	0	0	\N
2eb4d454-1a6a-47e1-8cc8-1283ca715128	\N	yumi		$argon2id$v=19$m=65536,t=3,p=4$quA8C2rreXbhlEPKE750RA$kUSbxxQ/qlMhomdUGMbYjBLC6SktHfulkHVuAEGXiA4	\N	\N	\N	User	1787794713	1787794713	0	0	\N
36e9e804-ed7a-45c6-bcf1-87daa36e74ca	\N	rosalita	rosalita@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$ufjbwdqaHIxnr4v4szyi0g$WRrwAtmAmWTSX3mE1tYrZeMLqMEdUr1flH8v9RUWnHA	\N	\N	\N	User	1785387739	1785387739	0	0	\N
3ec1cee0-0c96-4345-8c5e-93f2c4e726e7	\N	camallere	camallere@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$l4UTEnJHi6d1tjvkElCSaQ$fIYKbHE+9ONy5dTI73T/gOAMU9ShsbqstihlkF+Ty3Y	\N	\N	\N	User	1785312069	1785312069	0	0	\N
58337c4b-72ba-4229-bc30-e9f90f6efc14	\N	cacho	cacho@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$B6hUn4L6l6TnclOwabGWjw$GJnZi9xtA13wyyuZ/TdcGZ3LW1NboFfBepVFe/VbvAk	Cacho	Lourdes	Quinoo	Agent	1785216674	1785217024	0	0	\N
4f7d4ac8-9d67-40d8-9f44-23bdcc0ad903	\N	ibacarra	ibacarra@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$z7hV57YNGBIAyOvvAWeSJw$eDOqb7jWt2/NtTjDp7QswDW9q6Qt+3eL1XG/vLqf1s0	\N	\N	\N	User	1786252086	1786252086	0	0	\N
508ad9fd-c6d7-4ff6-b09c-ccf0cde4d6d8	\N	pascual	pascual@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$TP1SCNrp+S5zIQ3bCpcPPA$/o9g+Vgzq7kAaiIaMYmOFgyP9Kf0IJYJI8vhLlc5Gr4	\N	\N	\N	User	1785383062	1785383062	0	0	\N
53e14aaf-6ef2-428f-a506-17309494c9fa	\N	amor	amor@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$bwYhcwjgnzXkka7fFdxSNg$MwCI61GVqrHdQ+sZ+c8BFDJsthnGGd5yBF6uBmfm8zw	\N	\N	\N	User	1785312587	1785312587	0	0	\N
5bcc9fcc-0cca-44da-834a-20a307511776	\N	gumbason	gumbason@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$CGjJFzDyE63lGJ+l156lAQ$C2Nu5U8S7ubBoKUrMj/q6BzbSI3QNy9HrYqu5kOi3dI	\N	\N	\N	User	1785652268	1785652268	0	0	\N
604af0a9-ea54-44df-9af0-4c8e87705753	\N	marolina	marolina@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$7yjKHdqCllyvUVQRLwuD8g$w9lGGZLmam6JQLOlav54gd9GqKb4cJBucXqMng9GPhw	\N	\N	\N	User	1785386991	1785386991	0	0	\N
72ae543a-2c39-4259-a4e6-9c0068002b56	\N	jamesbond	jamesbond@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$qoTksjgg3aug+sLKGjSm7Q$WULBaQXQl3529fQuFPg9fu7uKv5LiRO0snKSXGwmsUQ	\N	\N	\N	User	1786545312	1786545312	0	0	\N
787c8487-9a54-411e-abc2-174cf4fa7c09	\N	algones	algones@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$VuotXuqdAA21zd2NypoCfA$Hnru91pKSebpF6Yy3Wf/Ysor5HcMO3eva9W+Cr0F/KI	\N	\N	\N	User	1785385020	1785385020	0	0	\N
81ce8e27-1745-4ed9-abe4-65bbc8ba8012	\N	adiao	adiao@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$aXZ3+7swJlVeVkSl7S7abg$9iGoVjfyJOqdba3ZPVwZB63dyYROlYQgcsBXP/OBGfc	\N	\N	\N	User	1785384622	1785384622	0	0	\N
8fcbfd05-41bc-4258-90d6-9ac255939824	\N	bermillo	bermillo@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$Ea7totABHdB0EXdvO3Zk9Q$LR28c6tb10Xrd9xvQv0R+MJ+t+bySafbH3u0iHbXSc8	\N	\N	\N	User	1785389154	1785389154	0	0	\N
904c9ced-a24b-4887-8d6e-01feda354383	\N	soriano	soriano@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$6DYa/AUcteK9rQf6y/dokQ$cA1lQb4y8RdR60zS1YUY6nqH+7K0B04gVetpmZJH3XA	\N	\N	\N	User	1785400578	1785400578	0	0	\N
9a81fb0c-f487-4b12-9102-8aa6ea22ecff	\N	fortich	fortich@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$zFDerNcgZRRkhfOhI5qu5w$SPj+2doUFcFzzUqoD68UeldPGK/06Z+2veEJNQhoNQs	\N	\N	\N	User	1785385980	1785385980	0	0	\N
bdf8af47-610b-4d69-b86f-29415b703d7d	\N	gemma	gemma@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$WFkTOMyNU6BkC9l/mJZ+5w$jnRNZvGVYj/CrNdk4EAaJavZSSHwnnmcCVhKJTNYO1Y	Suerto	Gemma	\N	Agent	1785312893	1785312948	0	0	\N
c0f45227-660a-4a76-b4da-98b19d781064	\N	thessaliehpropertyconsultancydavao	thessaliehpropertyconsultancydavao@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$S0vnkzn2Vyww+U2JHddWow$PQPeYkEl5XysQojnOVgbFbVuLV+u4O2lbZ2c1LNPjUg	\N	\N	\N	Admin	1785132482	1785132482	0	0	\N
d1206ad9-f01b-48e8-ac00-b7f4bcf4b63a	\N	florida	florida@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$R3dXD6fYUbPwOIWQ7uXM+A$F6AvzWdCyNAlteognlVoLrpSYZ05qeF+yt3NvFDPFi4	\N	\N	\N	User	1785223332	1785223332	0	0	\N
44c4948c-b76d-4523-8006-9581b036cea2	\N	gladez	gladez@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$ZZRHBcugIZdY10XQz7t7VA$+/uxJD8Dlafp0y9MFVtl6CD/AdhZIEw4jZFzKljlMlU	Mendoza	Gladez	\N	Lead Broker	1784807340	1784851111	0	0	\N
d5079571-1934-44eb-90f3-b448b560986a	\N	marliz		$argon2id$v=19$m=65536,t=3,p=4$RKQs+2AoBKZCLJNmvQtgcA$dgb4QVsk4YZOy0+i8T3s0EYICn9BxtjznzjDG5w16Nw	\N	\N	\N	Lead Broker	1787634622	1787723013	0	0	\N
7f258a62-7e0e-44a4-8cdf-0641aee2459c	\N	arcilie		$argon2id$v=19$m=65536,t=3,p=4$sQxywuxkFIyDttxdBH1NHg$Uqpd+3+cNGjVEbgMhndHry/JRtkS7ZW3LjpO9of5cTM	\N	\N	\N	Agent	1787722794	1787790306	0	0	\N
e84db191-2a42-4705-a97d-f78035dcd66b	\N	evelyn		$argon2id$v=19$m=65536,t=3,p=4$LFKxYqRO29KRqfseSeh//w$6hW2rqPbv2/JxYEL+uj5DyiuCF7o1vAhyKXCiOBpu5o	\N	\N	\N	Agent	1787722835	1787790361	0	0	\N
4a1fee23-718f-4677-8fd5-6d8f78463411	\N	rexey	rexey@gmail.com	$argon2id$v=19$m=65536,t=3,p=4$rs7Zfw5KqgYcuWIDJd4vsQ$qyjJtMRe17O7oFhN9Zr2Sk5VJBY+u4q9nDESx1UHBIk	\N	\N	\N	Normal Upline	1785379303	1787794367	0	0	\N
67abc61a-b515-4304-827f-2159114cc644	\N	glorifel		$argon2id$v=19$m=65536,t=3,p=4$JdIXWYjXlI+FBceyul8DEQ$BWbwbQBl/wlaCcJ8111Ik+phOd+zGm/ZRyEW1iN79II	\N	\N	\N	Agent	1787802405	1787803003	0	0	\N
\.


--
-- Data for Name: roster; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."roster" ("id", "company_id", "user_id", "role", "broker_id", "code", "prc_license_number", "commission_rate", "status", "created_at", "updated_at") FROM stdin;
d4308329-b467-4ebf-a828-e463b491656e	1	44c4948c-b76d-4523-8006-9581b036cea2	Lead Broker	\N	THS-0021	\N	0	Active	1784851111	1784851111
5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	1	c062e70d-7ef4-491c-8828-e1350f73ddc8	Titling Officer	\N	THS-0022	\N	0	Active	1784851132	1784851132
0edee6f7-5ab2-41b1-b754-3ebba9a2d0b5	1	58337c4b-72ba-4229-bc30-e9f90f6efc14	Agent	d4308329-b467-4ebf-a828-e463b491656e	THS-006001	\N	10	Active	1785217024	1785217024
8d058edb-98b1-4321-83b9-99fa79cd901d	1	bdf8af47-610b-4d69-b86f-29415b703d7d	Agent	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	THS-006002	\N	10	Active	1785312948	1785312948
bdba8dbd-cbe5-42bb-bb63-534317fff9b0	1	194a87ae-a361-410b-920d-1f5a2f9eb5ba	Agent	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	THS-006004	\N	10	Active	1785386225	1785386225
33b1d7b8-ab93-413b-86c7-8622cda2e636	1	a36ddd38-40c7-4c1a-8da7-cf490cbd36b0	Agent	d4308329-b467-4ebf-a828-e463b491656e	THS-006005	\N	10	Active	1785389639	1785389639
7aca5aa5-3c18-4791-9cfc-3487540f0f74	1	21665ba9-6be2-46db-8bd0-10df1d7d7822	Agent	d4308329-b467-4ebf-a828-e463b491656e	THS-006003	\N	10	Active	1785314792	1786006702
9abcaee6-8055-4e8c-ae9d-e8d84076ba8e	1	d5079571-1934-44eb-90f3-b448b560986a	Lead Broker	\N	BRKM-001	\N	0	Active	1787723013	1787723013
00c02c01-b2d4-4eb4-a06e-c9f33fcc81ee	1	7f258a62-7e0e-44a4-8cdf-0641aee2459c	Agent	9abcaee6-8055-4e8c-ae9d-e8d84076ba8e	THSM-006001	\N	14	Active	1787790306	1787790306
203feb59-4f69-4f14-803c-219197aab52f	1	e84db191-2a42-4705-a97d-f78035dcd66b	Agent	d4308329-b467-4ebf-a828-e463b491656e	THSM-006002	\N	12	Active	1787790361	1787790361
dc20e04b-6839-4f67-925a-63bbd97c3446	1	4a1fee23-718f-4677-8fd5-6d8f78463411	Normal Upline	\N	UPN-001	\N	0	Active	1787794367	1787794367
96feed84-64de-4773-ae90-7ff92daf93ed	1	67abc61a-b515-4304-827f-2159114cc644	Agent	dc20e04b-6839-4f67-925a-63bbd97c3446	THSM-0060034	\N	12	Active	1787803003	1787803003
\.


--
-- Data for Name: projects; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."projects" ("id", "company_id", "name", "created_at", "updated_at", "lead_broker_roster_id", "titling_officer_roster_id", "agent_commission_split_months", "agents_json", "subdivision_layout") FROM stdin;
f7323136-6005-47bb-bd40-289b402f2028	1	Manambulan - Villamor Village	1786594498	1787803004	9abcaee6-8055-4e8c-ae9d-e8d84076ba8e	\N	15	[{"id": "9abcaee6-8055-4e8c-ae9d-e8d84076ba8e", "name": "marliz", "role": "lead-broker", "nickname": "marliz", "parentId": null, "sharePercent": 0}, {"id": "00c02c01-b2d4-4eb4-a06e-c9f33fcc81ee", "name": "arcilie", "role": "downline", "nickname": "arcilie", "parentId": "9abcaee6-8055-4e8c-ae9d-e8d84076ba8e", "sharePercent": 14}, {"id": "203feb59-4f69-4f14-803c-219197aab52f", "name": "evelyn", "role": "downline", "nickname": "evelyn", "parentId": "00c02c01-b2d4-4eb4-a06e-c9f33fcc81ee", "sharePercent": 12}, {"id": "dc20e04b-6839-4f67-925a-63bbd97c3446", "name": "rexey", "role": "normal-upline", "nickname": "rexey", "parentId": null, "sharePercent": 0}, {"id": "96feed84-64de-4773-ae90-7ff92daf93ed", "name": "glorifel", "role": "downline", "nickname": "glorifel", "parentId": "dc20e04b-6839-4f67-925a-63bbd97c3446", "sharePercent": 12}]	\N
11111111-1111-4111-8111-111111111111	1	Tagakpan - Villamor Village	1783400697	1787803320	d4308329-b467-4ebf-a828-e463b491656e	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	15	[{"id": "d4308329-b467-4ebf-a828-e463b491656e", "name": "Mendoza Gladez", "role": "lead-broker", "nickname": "gladez", "parentId": null, "sharePercent": 0}, {"id": "5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1", "name": "Balani Rosa", "role": "titling-officer", "nickname": "rosa", "parentId": null, "sharePercent": 0}, {"id": "0edee6f7-5ab2-41b1-b754-3ebba9a2d0b5", "name": "Cacho Lourdes", "role": "downline", "nickname": "cacho", "parentId": "d4308329-b467-4ebf-a828-e463b491656e", "sharePercent": 10}, {"id": "8d058edb-98b1-4321-83b9-99fa79cd901d", "name": "Suerto Gemma", "role": "downline", "nickname": "gemma", "parentId": "5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1", "sharePercent": 10}, {"id": "7aca5aa5-3c18-4791-9cfc-3487540f0f74", "name": "Doldolia Jennifer", "role": "downline", "nickname": "jean", "parentId": "d4308329-b467-4ebf-a828-e463b491656e", "sharePercent": 10}, {"id": "bdba8dbd-cbe5-42bb-bb63-534317fff9b0", "name": "Calzada Monaliza", "role": "downline", "nickname": "calzada", "parentId": "5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1", "sharePercent": 10}, {"id": "33b1d7b8-ab93-413b-86c7-8622cda2e636", "name": "Doldolia Robert", "role": "downline", "nickname": "robert", "parentId": "d4308329-b467-4ebf-a828-e463b491656e", "sharePercent": 10}, {"id": "dc20e04b-6839-4f67-925a-63bbd97c3446", "name": "rexey", "role": "normal-upline", "nickname": "rexey", "parentId": null, "sharePercent": 0}, {"id": "96feed84-64de-4773-ae90-7ff92daf93ed", "name": "glorifel", "role": "downline", "nickname": "glorifel", "parentId": "dc20e04b-6839-4f67-925a-63bbd97c3446", "sharePercent": 11}, {"id": "203feb59-4f69-4f14-803c-219197aab52f", "name": "evelyn", "role": "downline", "nickname": "evelyn", "parentId": "dc20e04b-6839-4f67-925a-63bbd97c3446", "sharePercent": 10}]	\N
\.


--
-- Data for Name: agent_commission_period_status; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."agent_commission_period_status" ("id", "project_id", "period_key", "upline_role", "status", "partial_amount", "partial_note", "updated_at") FROM stdin;
\.


--
-- Data for Name: commission_period_status; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."commission_period_status" ("id", "project_id", "subject_agent_id", "period_start", "period_end", "status", "partial_amount", "partial_paid_at", "updated_at", "row_key", "paid_at") FROM stdin;
78bd28b6-ec23-4cb1-bc2e-8c9f86f0c01b	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-07-01	2026-07-15	not_yet	\N	\N	1785044462		\N
\.


--
-- Data for Name: commission_rates; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."commission_rates" ("role", "commission_rate", "updated_at") FROM stdin;
Agent	12	1784807582
Legal Counsel	5	1784809355
Land Owner	40	1784809355
Hypomone	25	1784809355
Project Dev & Processing	10	1784809355
Lead Broker	5	1787730561
Titling Officer	3	1787755758
\.


--
-- Data for Name: commission_release_credits; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."commission_release_credits" ("id", "project_id", "subject_agent_id", "share_kind", "amount", "paid_at", "note", "created_at") FROM stdin;
b27194ee-7f46-4b19-8b5c-19ac6fc6258f	11111111-1111-4111-8111-111111111111	7aca5aa5-3c18-4791-9cfc-3487540f0f74	\N	1900	2026-07-31	\N	1786432449
4c9f3f59-07d7-4a1b-ae58-e11c8dc388db	11111111-1111-4111-8111-111111111111	33b1d7b8-ab93-413b-86c7-8622cda2e636	\N	7.5	2026-07-31	\N	1786432809
\.


--
-- Data for Name: commission_release_entries; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."commission_release_entries" ("id", "project_id", "subject_agent_id", "period_start", "period_end", "amount", "paid_at", "created_at", "share_kind") FROM stdin;
b37fcc2d-ce52-40ce-b27a-264e812117a2	11111111-1111-4111-8111-111111111111	7aca5aa5-3c18-4791-9cfc-3487540f0f74	2026-06-16	2026-06-30	7100	2026-07-10	1786421824	\N
a5d130ec-510f-4bd0-80d4-61992114de27	11111111-1111-4111-8111-111111111111	7aca5aa5-3c18-4791-9cfc-3487540f0f74	2026-07-16	2026-07-31	4000	2026-08-22	1786423514	\N
e1b6eb16-5ffb-4421-b060-34e79488ebb7	11111111-1111-4111-8111-111111111111	7aca5aa5-3c18-4791-9cfc-3487540f0f74	2026-07-16	2026-07-31	3100	2026-07-31	1786432448	\N
9c543330-d33d-4379-b516-ea1869277541	11111111-1111-4111-8111-111111111111	33b1d7b8-ab93-413b-86c7-8622cda2e636	2026-07-01	2026-07-15	4000	2026-07-22	1786432741	\N
60dbe667-1810-4eb8-9e9b-4a456e00b3d5	11111111-1111-4111-8111-111111111111	33b1d7b8-ab93-413b-86c7-8622cda2e636	2026-07-01	2026-07-15	2842.5	2026-07-31	1786432808	\N
1471435c-2bf3-4d30-8c93-d9f033beb046	11111111-1111-4111-8111-111111111111	33b1d7b8-ab93-413b-86c7-8622cda2e636	2026-07-16	2026-07-31	2150	2026-07-31	1786432808	\N
9b9bd228-7bb9-476f-81fa-c628f965cd2f	11111111-1111-4111-8111-111111111111	0edee6f7-5ab2-41b1-b754-3ebba9a2d0b5	2026-06-01	2026-06-15	2587.5	2026-06-19	1786433793	\N
f9a92dec-cc7a-4df5-9fd6-093435d9fe5b	11111111-1111-4111-8111-111111111111	0edee6f7-5ab2-41b1-b754-3ebba9a2d0b5	2026-07-16	2026-07-31	2587.5	2026-08-08	1786433894	\N
c413a60e-757f-41d0-9da7-2bdaa9c408e7	11111111-1111-4111-8111-111111111111	8d058edb-98b1-4321-83b9-99fa79cd901d	2026-06-16	2026-06-30	4000	2026-07-16	1786434169	\N
2faaf4fb-de7e-4da9-b120-99d7326e2a6f	11111111-1111-4111-8111-111111111111	8d058edb-98b1-4321-83b9-99fa79cd901d	2026-06-16	2026-06-30	2965	2026-07-31	1786434189	\N
8420c78b-3035-4b9c-aa19-b59fcb0e49e4	11111111-1111-4111-8111-111111111111	8d058edb-98b1-4321-83b9-99fa79cd901d	2026-07-01	2026-07-15	2035	2026-07-31	1786434189	\N
3a33b8b7-808a-4755-85fe-f13eeaed6df5	11111111-1111-4111-8111-111111111111	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	2026-07-01	2026-07-15	5000	2026-07-16	1786435109	\N
7465473d-0c27-43a9-acb6-b11c10a73f48	11111111-1111-4111-8111-111111111111	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	2026-07-01	2026-07-15	5462	2026-08-01	1786435135	\N
26339728-9ee9-4522-a76a-56b9a1328e24	11111111-1111-4111-8111-111111111111	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	2026-06-01	2026-06-15	3357	2026-07-16	1786437654	base
ce50b820-6ca1-4802-85f2-b6eaaafa7db4	11111111-1111-4111-8111-111111111111	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	2026-06-16	2026-06-30	2643	2026-07-16	1786437655	base
6349fe62-ed69-4e7b-91fe-e5c9c6d0e0da	11111111-1111-4111-8111-111111111111	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	2026-06-16	2026-06-30	5000	2026-08-08	1786437712	base
fc3a647d-38aa-4332-8c3b-3905c961ba4f	11111111-1111-4111-8111-111111111111	8d058edb-98b1-4321-83b9-99fa79cd901d	2026-07-01	2026-07-15	15000	2026-07-08	1786844750	promo
9c900018-348e-4bf1-b7dd-9f1164b7eb34	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-06-01	2026-06-15	7833	2026-06-18	1786860096	base
b7cc9f6e-43b9-45aa-a84e-5b441ce60b86	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-06-16	2026-06-30	10364.5	2026-06-18	1786860096	base
90c971e2-e6fd-41db-a848-6515eaa46e97	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-06-16	2026-06-30	3726.5	2026-07-21	1786860285	base
1dee0756-20fe-4cdf-9256-1bb191b39827	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-07-01	2026-07-15	6273.5	2026-07-21	1786860285	base
c0cc1307-f778-410e-be7a-9dbe80ca0ac0	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-07-01	2026-07-15	5000	2026-08-04	1786860338	base
4b30c6c5-0ad9-4b60-b4e9-411650b2f68b	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-08-16	2026-08-31	4900	2026-08-27	1787818971	promo
db1a1827-b05e-4625-8d83-faf9b2af2d71	11111111-1111-4111-8111-111111111111	33b1d7b8-ab93-413b-86c7-8622cda2e636	2026-08-16	2026-08-31	7000	2026-08-27	1787819013	promo
3a6a96d0-8e04-4ae2-a328-f34bc94a7f03	11111111-1111-4111-8111-111111111111	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	2026-08-16	2026-08-31	2100	2026-08-27	1787819037	promo
f9e55558-865c-4969-800c-fa2addc44715	11111111-1111-4111-8111-111111111111	8d058edb-98b1-4321-83b9-99fa79cd901d	2026-08-16	2026-08-31	1474.14	2026-08-27	1787820306	\N
bcd6bc78-597f-4fc0-8889-7c1d03217c11	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-08-16	2026-08-31	750	2026-08-27	1787820398	base
ca289e3d-1e81-45ba-bd4f-663afe56c700	11111111-1111-4111-8111-111111111111	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	2026-08-16	2026-08-31	750	2026-08-27	1787820444	base
fa513b50-cd97-411c-a57c-a60fc04956ef	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-08-16	2026-08-31	990	2026-08-27	1787874875	base
99924f5d-6c04-43fd-88b7-178e7291ca7d	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-08-16	2026-08-31	2376	2026-08-27	1787874898	pool
1432196c-aaea-4a7a-9e3f-8f5161b735f6	11111111-1111-4111-8111-111111111111	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	2026-08-16	2026-08-31	594	2026-08-27	1787874991	base
963d4d55-a65f-42be-a300-4a173c56492c	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-08-16	2026-08-31	2013.43	2026-08-28	1788057129	pool
4f2f473f-251a-4338-991e-0b60819548d5	11111111-1111-4111-8111-111111111111	5b360bf3-59cb-43e2-bb0e-7d3b566e7bb1	2026-08-16	2026-08-31	503.36	2026-08-28	1788057150	base
70adb83a-4138-4903-ab34-ff5abb796324	11111111-1111-4111-8111-111111111111	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	2026-08-16	2026-08-31	1562.59	2026-08-30	1788174504	\N
931c1ec1-f8ac-4e88-8a60-c334433b66f6	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	2026-08-16	2026-08-31	781.29	2026-08-30	1788174552	base
\.


--
-- Data for Name: commission_row_meta; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."commission_row_meta" ("id", "project_id", "subject_agent_id", "row_key", "period_start", "other_flag", "updated_at", "override_amount") FROM stdin;
3d5f83fb-1ef9-4af9-9f53-b2219f36c9bf	11111111-1111-4111-8111-111111111111	d4308329-b467-4ebf-a828-e463b491656e	d4308329-b467-4ebf-a828-e463b491656e	2026-07-01	none	1785329739	\N
\.


--
-- Data for Name: commission_split_schedule; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."commission_split_schedule" ("id", "effective_date", "split_months", "created_at", "updated_at") FROM stdin;
18229a71-e1ea-4869-8f4f-e5dfdc940f09	2000-01-01	36	1785129377	1785129377
\.


--
-- Data for Name: lots; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."lots" ("id", "project_id", "block", "lot", "lot_type", "area", "rate", "contract_price", "owner_buyer", "on_hold", "status", "created_at", "updated_at", "reserved_until", "reserve_notes", "reserve_agent_id") FROM stdin;
f8e5c736-46e9-43fd-9289-bf12d8c50a1c	11111111-1111-4111-8111-111111111111	Block 1	1	Commercial / Corner	100	4500	450000	\N	f	Available	1785213957	1785213957	\N		\N
1680f609-71f7-4810-b718-358fc81a0ac8	11111111-1111-4111-8111-111111111111	Block 1	3	Commercial / Corner	115	4500	517500	\N	f	Available	1785213957	1785213957	\N		\N
6bb5dfe1-897a-4902-94a9-917120ef20a1	11111111-1111-4111-8111-111111111111	Block 1	4	Commercial	112	4500	504000	\N	f	Available	1785213957	1785213957	\N		\N
6b42b985-3975-4162-8eaf-d9b27188f4b1	11111111-1111-4111-8111-111111111111	Block 1	5	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
cdc781d5-049c-4998-8db6-a4f2ef3da162	11111111-1111-4111-8111-111111111111	Block 1	10	Commercial	115	4500	517500	Cacho, Lourdes Quinoo	f	Installment	1785213957	1785213957	\N		\N
200c0c79-a53a-40ca-a9aa-9de0d4de18e7	11111111-1111-4111-8111-111111111111	Block 1	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
e7877306-60ca-4ce7-9e6a-a5e8360306ed	11111111-1111-4111-8111-111111111111	Block 1	11	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
45a78fe1-d18e-4044-a54d-67d3afa84294	11111111-1111-4111-8111-111111111111	Block 1	2	Inner	100	4300	430000	\N	f	Reserved	1785213957	1786342900	2026-09-14 06:21:40+00	Reserve by ate Dalc	\N
2c0ea7b7-a989-4fd1-8ea3-88e88b89477c	11111111-1111-4111-8111-111111111111	Block 1	15	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
7876c84e-832b-43f0-a24a-32f196ba23c1	11111111-1111-4111-8111-111111111111	Block 2	2	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
7a22a607-c2b7-4742-a0c9-d1b41628b21f	11111111-1111-4111-8111-111111111111	Block 2	3	Commercial / Corner	100	4500	450000	\N	f	Available	1785213957	1785213957	\N		\N
37662da9-b73f-4c64-8079-0813b0ce0b3a	11111111-1111-4111-8111-111111111111	Block 2	4	Commercial	107	4500	481500	\N	f	Available	1785213957	1785213957	\N		\N
30ca638e-0c3a-444c-821b-5665cd555dab	11111111-1111-4111-8111-111111111111	Block 2	7	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
9f2a7f83-5bc9-4484-ba0a-7c76b0b09140	11111111-1111-4111-8111-111111111111	Block 2	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
6b68fc7e-1cf7-4097-80a0-d277d0f7eb20	11111111-1111-4111-8111-111111111111	Block 2	10	Commercial	104	4500	468000	\N	f	Available	1785213957	1785213957	\N		\N
31cdcf37-d326-42b5-ab65-baf6359e7cd6	11111111-1111-4111-8111-111111111111	Block 2	11	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
1b5294bb-28ec-4bc7-9ec1-621d9b29f599	11111111-1111-4111-8111-111111111111	Block 2	12	Commercial	103	4500	463500	\N	f	Available	1785213957	1785213957	\N		\N
bab7809d-3ccd-462b-a5a9-4dc334a91b8f	11111111-1111-4111-8111-111111111111	Block 2	13	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
fdc4c4f9-61e4-4a37-8ac2-bface8b160fe	11111111-1111-4111-8111-111111111111	Block 2	14	Commercial	102	4500	459000	\N	f	Available	1785213957	1785213957	\N		\N
5ed5c555-43d0-4c2a-8edb-56aacc0c478d	11111111-1111-4111-8111-111111111111	Block 2	15	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
fe8e3ba8-8cc2-4be3-90bc-bd855670eed1	11111111-1111-4111-8111-111111111111	Block 2	17	Corner	143	4500	643500	\N	f	Available	1785213957	1785213957	\N		\N
2b0a13fa-cd47-4899-b481-0b6ea3974609	11111111-1111-4111-8111-111111111111	Block 3	1	Corner	121	4500	544500	\N	f	Available	1785213957	1785213957	\N		\N
5786b376-7338-4c52-a6a4-108f52a33327	11111111-1111-4111-8111-111111111111	Block 3	2	Commercial / Corner	100	4500	450000	\N	f	Available	1785213957	1785213957	\N		\N
97ae2d3d-c227-4625-b13e-eef1d45ed128	11111111-1111-4111-8111-111111111111	Block 3	3	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
38624253-3a3b-4ba4-ac73-8fff16ba2f1e	11111111-1111-4111-8111-111111111111	Block 3	4	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
3e2bcb76-abb9-431a-a887-382ade1a4b2b	11111111-1111-4111-8111-111111111111	Block 3	5	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
97f15219-b439-45b7-a903-b87c5b0c9cee	11111111-1111-4111-8111-111111111111	Block 3	6	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
73ea7489-9fe7-40f8-861b-4a56bfb2b70c	11111111-1111-4111-8111-111111111111	Block 3	7	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
dfe60e7f-9028-4194-a3af-bca7fb6cffab	11111111-1111-4111-8111-111111111111	Block 3	8	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
4d4597ad-18e9-4c1c-b92e-f13199ff45ad	11111111-1111-4111-8111-111111111111	Block 3	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
d6a00174-17cc-4e71-9652-5d9193c02b25	11111111-1111-4111-8111-111111111111	Block 3	10	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
77b59047-fa82-476e-a642-2f7d6a5d06ca	11111111-1111-4111-8111-111111111111	Block 3	11	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
7359b1e6-05ee-493e-af45-b564a29f16f4	11111111-1111-4111-8111-111111111111	Block 3	12	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
fdc39ba7-0ded-4449-891d-7019a68e3fe6	11111111-1111-4111-8111-111111111111	Block 3	13	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
a5264325-986c-4ad0-b3d7-f28eafda2c2e	11111111-1111-4111-8111-111111111111	Block 4	4	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
a67d195a-7ab5-45c1-a449-d000d187da67	11111111-1111-4111-8111-111111111111	Block 4	5	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
4e23863c-3cc3-4ea7-94dc-4cff80e4f9de	11111111-1111-4111-8111-111111111111	Block 4	6	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
f5ab6379-0930-420b-ab02-77be31886dd4	11111111-1111-4111-8111-111111111111	Block 4	7	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
0e800255-a1b2-4d06-8b32-e27222381758	11111111-1111-4111-8111-111111111111	Block 4	8	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
51d730c0-9170-44a5-82de-2d79869f7cdb	11111111-1111-4111-8111-111111111111	Block 4	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
87a8f35c-c63b-4477-9eaa-1d3cdb5fc032	11111111-1111-4111-8111-111111111111	Block 4	10	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
a8fe72ac-96aa-46f8-945d-b6b14deca223	11111111-1111-4111-8111-111111111111	Block 4	11	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
4acfbb01-1966-4618-844d-0e27a1f16978	11111111-1111-4111-8111-111111111111	Block 4	12	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
ca3e94b8-da30-414e-9514-decfef3d9180	11111111-1111-4111-8111-111111111111	Block 4	13	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
e727825f-7589-4585-8bda-6358e7d7605e	11111111-1111-4111-8111-111111111111	Block 4	14	Corner	132	4500	594000	\N	f	Available	1785213957	1785213957	\N		\N
88d70503-a9ca-48d2-8754-dfeb2cdbac65	11111111-1111-4111-8111-111111111111	Block 4	15	Corner	110	4500	495000	\N	f	Available	1785213957	1785213957	\N		\N
12f1da1e-fb93-480e-b582-04b8b4a119ba	11111111-1111-4111-8111-111111111111	Block 6	13	Inner	100	4300	430000	\N	f	Reserved	1785213957	1788055647	2026-09-06 02:07:29+00	Yumi split 3 months	dc20e04b-6839-4f67-925a-63bbd97c3446
2dd0b6d4-9198-42b7-a5dd-6960601dd527	11111111-1111-4111-8111-111111111111	Block 5	3	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
2d9a33f7-0623-40a8-a927-dbd84678acca	11111111-1111-4111-8111-111111111111	Block 6	3	Inner	100	4300	430000	\N	f	Reserved	1785213957	1787488231	2026-12-21 12:30:31+00	Reserve by Ate	\N
a0744d49-af17-489e-b4a1-574eea02e0bc	11111111-1111-4111-8111-111111111111	Block 5	5	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
227ae480-86d5-4387-86f4-fdae8fe4b0c7	11111111-1111-4111-8111-111111111111	Block 6	4	Inner	100	4300	430000	\N	f	Reserved	1785213957	1787488231	2026-12-21 12:30:31+00	Reserve by Ate	\N
961b5dbf-b50b-4687-8731-45abca597a23	11111111-1111-4111-8111-111111111111	Block 6	11	Inner	100	4300	430000	\N	f	Reserved	1785213957	1788055708	2026-09-06 02:08:30+00	Yum - spot cash	dc20e04b-6839-4f67-925a-63bbd97c3446
e67865ac-8702-407b-861c-25330a6f912e	11111111-1111-4111-8111-111111111111	Block 6	5	Inner	100	4300	430000	\N	f	Reserved	1785213957	1787488231	2026-12-21 12:30:31+00	Reserve by Ate	\N
64439c8c-c989-4061-b91e-8504192059f4	11111111-1111-4111-8111-111111111111	Block 6	6	Inner	100	4300	430000	\N	f	Reserved	1785213957	1787488231	2026-12-21 12:30:31+00	Reserve by Ate	\N
5ef1160c-0138-409d-92ff-72060135045f	11111111-1111-4111-8111-111111111111	Block 5	10	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
0dc66e90-cf04-49e1-896d-bb0c143639cc	11111111-1111-4111-8111-111111111111	Block 5	11	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
46b41e91-0c6f-4e17-b8f5-87d06f86bd25	11111111-1111-4111-8111-111111111111	Block 5	12	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
4639abcb-d975-4531-a6f6-2e2e7b578056	11111111-1111-4111-8111-111111111111	Block 5	13	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
6831c20c-ebbb-4f7c-b071-33a148cf7a96	11111111-1111-4111-8111-111111111111	Block 5	14	Corner	132	4500	594000	\N	f	Available	1785213957	1785213957	\N		\N
3fdc8407-d3d4-454d-98c8-71ff80eb6287	11111111-1111-4111-8111-111111111111	Block 5	15	Corner	110	4500	495000	\N	f	Available	1785213957	1785213957	\N		\N
1d47e84b-becf-4e3b-869a-0df6a1111921	11111111-1111-4111-8111-111111111111	Block 6	7	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
f293fea4-e549-453b-8702-671960455804	11111111-1111-4111-8111-111111111111	Block 6	8	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
8d3b64a4-7692-4851-87fe-50a83b249ab9	11111111-1111-4111-8111-111111111111	Block 6	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
f944d4e4-9e2a-4151-a7c4-e001a1895ac0	11111111-1111-4111-8111-111111111111	Block 6	10	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
8c6ccef2-c1a7-4859-bce4-f3e1a72d9698	11111111-1111-4111-8111-111111111111	Block 6	12	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
82908a40-efe7-4455-9ac4-e3e9e226f045	11111111-1111-4111-8111-111111111111	Block 6	14	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
16dc9923-74b9-408d-b4a3-39eabf9c2337	11111111-1111-4111-8111-111111111111	Block 6	15	Corner	100	4500	450000	\N	f	Available	1785213957	1785213957	\N		\N
2a28e63c-da61-4336-aae2-abf5d98ee0b9	11111111-1111-4111-8111-111111111111	Block 6	16	Inner	117	4300	503100	\N	f	Available	1785213957	1785213957	\N		\N
a80c5e63-cc9c-44f6-bb42-64266a6ea5ff	11111111-1111-4111-8111-111111111111	Block 6	17	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
d5f6d9dd-6d79-4092-b89a-1107b451e775	11111111-1111-4111-8111-111111111111	Block 6	18	Corner	113	4500	508500	\N	f	Available	1785213957	1785213957	\N		\N
24592255-dfd4-4c3a-ac38-b1a432cfc6a3	11111111-1111-4111-8111-111111111111	Block 7	1	Commercial / Corner	100	4500	450000	\N	f	Available	1785213957	1785213957	\N		\N
c0f7bb54-2ff0-4ba7-a36b-da06e7c97145	11111111-1111-4111-8111-111111111111	Block 7	2	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
de7e4cf5-809c-431d-9813-f3457389f0fa	11111111-1111-4111-8111-111111111111	Block 7	3	Commercial / Corner	155	4500	697500	\N	f	Available	1785213957	1785213957	\N		\N
66644223-2467-47e9-88cf-95b87a2e05fc	11111111-1111-4111-8111-111111111111	Block 7	4	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
b9c403d2-1a59-4d21-9800-c2ac60b7caa4	11111111-1111-4111-8111-111111111111	Block 7	5	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
1779ba6a-edcc-4bbf-8af1-270ba2bb7ef3	11111111-1111-4111-8111-111111111111	Block 7	6	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
1bc0e04d-3888-4d97-96ba-d0a296183562	11111111-1111-4111-8111-111111111111	Block 7	7	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
78587438-089c-4f60-825c-840e886d382f	11111111-1111-4111-8111-111111111111	Block 7	8	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
3f86fb37-d203-49e2-92fd-8585e8e53bc7	11111111-1111-4111-8111-111111111111	Block 6	1	Corner	116	4500	522000	Lumingkit, Ariel Arnais	f	Installment	1785213957	1785213957	\N		\N
0f51afa9-12e3-4d17-8565-7a7885d2d277	11111111-1111-4111-8111-111111111111	Block 7	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
7d0d507e-0fcc-450b-9831-d2b6270eee0c	11111111-1111-4111-8111-111111111111	Block 7	10	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
d6a61202-4d0d-41a8-88bd-2b822e0e5585	11111111-1111-4111-8111-111111111111	Block 7	11	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
21a66289-84e4-4e3c-8562-3da81090f893	f7323136-6005-47bb-bd40-289b402f2028	3	5	Inner	100	3500	350000	\N	f	Available	1787721908	1787721908	\N		\N
80599549-f8eb-4ceb-b312-41a3af32a423	11111111-1111-4111-8111-111111111111	Block 7	12	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
1eea4fcc-114e-412f-a8ec-223ce784baa6	11111111-1111-4111-8111-111111111111	Block 7	13	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
2afb6dc1-b9f7-48a2-a15f-21863e54215e	11111111-1111-4111-8111-111111111111	Block 7	14	Commercial / Corner	110	4500	495000	\N	f	Available	1785213957	1785213957	\N		\N
ce68b995-1b97-46e6-82d7-db60e2a38a1c	11111111-1111-4111-8111-111111111111	Block 7	15	Commercial / Corner	144	4500	648000	\N	f	Available	1785213957	1785213957	\N		\N
05c0e88a-aeb9-4ec6-bbb6-5bca2adc15cf	11111111-1111-4111-8111-111111111111	Block 8	1	Commercial / Corner	134	4500	603000	\N	f	Available	1785213957	1785213957	\N		\N
987f6e8e-b7f9-406d-9540-0720bdbf653a	11111111-1111-4111-8111-111111111111	Block 8	2	Commercial / Corner	100	4500	450000	\N	f	Available	1785213957	1785213957	\N		\N
dcef4d68-6f31-4a6d-a122-a2c2493a7531	11111111-1111-4111-8111-111111111111	Block 8	3	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
4e08f21e-14c2-403c-a68c-9e777aea92e5	11111111-1111-4111-8111-111111111111	Block 8	4	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
ee0eb6e5-804f-474d-b12c-29290c9b415d	11111111-1111-4111-8111-111111111111	Block 8	5	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
a30f935d-5b6a-4d16-bfb4-db44882bce38	f7323136-6005-47bb-bd40-289b402f2028	3	6	Inner	100	3500	350000	\N	f	Available	1787721926	1787721926	\N		\N
e05dbdc1-e177-4c52-b911-9fd752d205df	11111111-1111-4111-8111-111111111111	Block 8	6	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
651d2abf-f73b-4ca8-95cc-c0f8b2289ea8	11111111-1111-4111-8111-111111111111	Block 8	7	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
36d5c9db-9eb1-40f2-aeb9-9ca296b87be7	11111111-1111-4111-8111-111111111111	Block 8	8	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
6e583777-380b-48ed-ac0d-ded0ea6bc70d	11111111-1111-4111-8111-111111111111	Block 8	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
cd976336-66e4-4e20-aa91-0f923f427dae	11111111-1111-4111-8111-111111111111	Block 8	10	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
fe4b38d1-0ec4-482a-9fa5-511354c9c896	11111111-1111-4111-8111-111111111111	Block 8	11	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
c0b93f48-fba2-470e-833f-dacef810f617	11111111-1111-4111-8111-111111111111	Block 8	12	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
4d55bb19-9e2b-4bd3-b19d-dddfe7d2d059	11111111-1111-4111-8111-111111111111	Block 8	13	Commercial / Corner	164	4500	738000	\N	f	Available	1785213957	1785213957	\N		\N
fab5730f-c540-4b5b-9910-3b11accf5e26	11111111-1111-4111-8111-111111111111	Block 8	14	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
4d446ecb-c398-4e6c-b821-7b9e262d3964	11111111-1111-4111-8111-111111111111	Block 8	15	Commercial / Corner	111	4500	499500	\N	f	Available	1785213957	1785213957	\N		\N
e07502dd-cd92-434e-9b17-1aba7fb110e4	f7323136-6005-47bb-bd40-289b402f2028	3	9	Inner	100	3500	350000	\N	f	Available	1787721965	1787721965	\N		\N
81982ad4-fb77-47f9-826a-4219444f9889	f7323136-6005-47bb-bd40-289b402f2028	4	2	Inner	100	3500	350000	\N	f	Available	1787722036	1787722036	\N		\N
49cd1b25-93f9-495f-933e-362dc8c2076f	f7323136-6005-47bb-bd40-289b402f2028	4	3	Inner	100	3500	350000	\N	f	Available	1787722052	1787722052	\N		\N
2241a991-8dd8-4195-806a-9be204df1b31	11111111-1111-4111-8111-111111111111	Block 5	4	Inner	100	4300	430000	Algones, Ejea	f	Installment	1785213957	1785213957	\N		\N
af5a2877-7eef-4fc8-b3a8-63639cfd9a8f	11111111-1111-4111-8111-111111111111	Block 1	6	Commercial	113	4500	508500	Bermillo, Jorgeanne Barbarona	f	Installment	1785213957	1785213957	\N		\N
eece7af0-6517-46fa-8499-0b659445996b	f7323136-6005-47bb-bd40-289b402f2028	1	1	Corner	122	3800	463600	\N	f	Available	1787721284	1787721284	\N		\N
744f4b33-0151-48ee-879d-b09d5c870856	f7323136-6005-47bb-bd40-289b402f2028	1	7	Inner	100	3500	350000	\N	f	Available	1787721419	1787721419	\N		\N
694fbb07-8f92-4181-a95e-0c8f0f44894f	f7323136-6005-47bb-bd40-289b402f2028	1	8	Inner	100	3500	350000	\N	f	Available	1787721438	1787721438	\N		\N
405a1105-b12c-4372-8ac6-28b80e8ea0c4	f7323136-6005-47bb-bd40-289b402f2028	2	5	Inner	115	3500	402500	\N	f	Available	1787721644	1787721644	\N		\N
89e4ab46-9c28-447f-9c82-f46638428147	f7323136-6005-47bb-bd40-289b402f2028	2	8	Inner	115	3500	402500	\N	f	Available	1787721731	1787721731	\N		\N
0b7162f9-7a2f-465f-a6c0-e67f18c32071	f7323136-6005-47bb-bd40-289b402f2028	3	1	Corner	124	3800	471200	\N	f	Available	1787721772	1787721772	\N		\N
4911c421-5871-41d5-9b4d-75feed3d479b	11111111-1111-4111-8111-111111111111	Block 1	14	Commercial	117	4500	526500	Florida, Wilmie	f	Installment	1785213957	1785213957	\N		\N
c0f12af6-6494-47ff-8809-24beea8a6a1e	11111111-1111-4111-8111-111111111111	Block 5	1	Corner	119	4500	535500	Marolina, Zaryll Praise	f	Installment	1785213957	1785213957	\N		\N
555735f4-6752-4c54-9cc2-d239537bb425	f7323136-6005-47bb-bd40-289b402f2028	1	2	Corner	120	3800	456000	\N	f	Available	1787721316	1787721316	\N		\N
228a2894-7f12-47b7-bbb3-ad664ab46fef	f7323136-6005-47bb-bd40-289b402f2028	1	5	Inner	100	3500	350000	\N	f	Available	1787721376	1787721376	\N		\N
23f4ffd9-13d6-413d-8789-f8649e7d403b	f7323136-6005-47bb-bd40-289b402f2028	1	6	Inner	100	3500	350000	\N	f	Available	1787721402	1787721402	\N		\N
8eae5882-b1aa-4868-b786-b917e046deb7	f7323136-6005-47bb-bd40-289b402f2028	1	9	Inner	100	3500	350000	\N	f	Available	1787721459	1787721459	\N		\N
7a583d93-262a-48b5-8242-f9bf52ce80e2	f7323136-6005-47bb-bd40-289b402f2028	1	10	Inner	100	3500	350000	\N	f	Available	1787721475	1787721475	\N		\N
1f42ef2c-0b4f-48b2-a91e-c6fdedf98428	f7323136-6005-47bb-bd40-289b402f2028	2	2	Corner	124	3800	471200	\N	f	Available	1787721545	1787721545	\N		\N
d3836c3f-aee6-4fff-8b7c-ed06c8660579	f7323136-6005-47bb-bd40-289b402f2028	2	3	Inner	115	3500	402500	\N	f	Available	1787721589	1787721589	\N		\N
6d9dd47e-32bb-4a7f-a76a-df90466f7d01	f7323136-6005-47bb-bd40-289b402f2028	2	6	Inner	115	3500	402500	\N	f	Available	1787721663	1787721663	\N		\N
66fbf572-f46e-40bb-ab6c-7c0387f3b725	f7323136-6005-47bb-bd40-289b402f2028	2	7	Inner	115	3500	402500	\N	f	Available	1787721711	1787721711	\N		\N
fd7a8c4d-3b9b-4aaa-834c-885a8b8b1d71	f7323136-6005-47bb-bd40-289b402f2028	3	8	Inner	100	3500	350000	\N	f	Available	1787721953	1787721953	\N		\N
fefdac9c-7579-4ba4-afd9-0838334e27dd	f7323136-6005-47bb-bd40-289b402f2028	4	1	Corner	104	3800	395200	\N	f	Available	1787722027	1787722027	\N		\N
08ecf50c-1d32-4205-bf0f-cdbadf7229a7	f7323136-6005-47bb-bd40-289b402f2028	4	4	Inner	100	3500	350000	\N	f	Available	1787722066	1787722066	\N		\N
5ecc01e9-3ee7-40c9-b675-b8b150352848	f7323136-6005-47bb-bd40-289b402f2028	4	5	Inner	100	3500	350000	\N	f	Available	1787722077	1787722077	\N		\N
d34f9cf6-d83a-4f0c-bec1-c603851ff8c2	f7323136-6005-47bb-bd40-289b402f2028	1	3	Inner	100	3500	350000	\N	f	Available	1787721340	1787721340	\N		\N
0cf4e2f1-5e91-4630-a4b0-ac00481ab35b	f7323136-6005-47bb-bd40-289b402f2028	1	4	Inner	100	3500	350000	\N	f	Available	1787721358	1787721358	\N		\N
2f70b944-a457-4f88-bd04-8e4aa1de893a	f7323136-6005-47bb-bd40-289b402f2028	2	1	Corner	124	3800	471200	\N	f	Available	1787721521	1787721521	\N		\N
02f24272-106f-4126-83c6-a438ed9c128f	11111111-1111-4111-8111-111111111111	Block 5	6	Inner	100	4300	430000	Soriano, Julius Aldemita	f	Installment	1785213957	1785213957	\N		\N
a97c030f-e8d4-437d-b2d7-c88481de6e40	f7323136-6005-47bb-bd40-289b402f2028	2	4	Inner	115	3500	402500	\N	f	Available	1787721619	1787721619	\N		\N
131d718b-92a3-4763-8bf0-4686ed70dfca	11111111-1111-4111-8111-111111111111	Block 5	7	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
177f3293-57f7-457c-a5bd-58403302329d	11111111-1111-4111-8111-111111111111	Block 5	8	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
69f0637c-442f-456d-9e67-0998070e70e4	11111111-1111-4111-8111-111111111111	Block 5	9	Inner	100	4300	430000	\N	f	Available	1785213957	1785213957	\N		\N
1703885a-39d9-4158-81e6-2574bfe8a6c1	11111111-1111-4111-8111-111111111111	Block 2	16	Commercial / Corner	100	4500	450000	Dongiapon, Jenilyn Pagapong	f	Installment	1785213957	1785213957	\N		\N
a9741975-e01d-4bd3-b0bf-fdf9602f2570	f7323136-6005-47bb-bd40-289b402f2028	3	2	Corner	122	3800	463600	\N	f	Available	1787721793	1787721793	\N		\N
11fbc445-4cf1-4061-a5c6-fbe3cb7e0be9	f7323136-6005-47bb-bd40-289b402f2028	3	3	Inner	100	3500	350000	\N	f	Available	1787721828	1787721828	\N		\N
a785e836-9962-46b5-8b23-639831f95873	f7323136-6005-47bb-bd40-289b402f2028	3	4	Inner	100	3500	350000	\N	f	Available	1787721886	1787721886	\N		\N
eb7ba35a-ea74-4636-89ab-cd6c8c5f27bf	11111111-1111-4111-8111-111111111111	Block 1	16	Commercial / Corner	157	4500	706500	Suerto, Gemma	f	Installment	1785213957	1785213957	\N		\N
836e822e-5aec-407b-8e75-cdc088a277e1	f7323136-6005-47bb-bd40-289b402f2028	3	7	Inner	100	3500	350000	\N	f	Available	1787721939	1787721939	\N		\N
b3463521-e0da-45e4-8fb3-e0dfdb64b602	f7323136-6005-47bb-bd40-289b402f2028	3	10	Inner	100	3500	350000	\N	f	Available	1787721985	1787721985	\N		\N
d7279f30-fd32-471e-935f-688ae12d8852	11111111-1111-4111-8111-111111111111	Block 1	13	Inner	100	4300	430000	Marces, Erwin	f	Installment	1785213957	1785213957	\N		\N
86663e2f-a8cd-4864-95b2-5055457a9ee8	11111111-1111-4111-8111-111111111111	Block 4	3	Inner	100	4300	430000	Escalera, Lorie Ramos	f	Installment	1785213957	1785213957	\N		\N
f060546c-f9ce-41f8-a6ba-d7ef2a0228e2	11111111-1111-4111-8111-111111111111	Block 4	2	Corner	100	4500	450000	Escalera, Lorie Ramos	f	Installment	1785213957	1785213957	\N		\N
981e0794-a981-42e2-a2a1-93706e6f3a50	11111111-1111-4111-8111-111111111111	Block 4	1	Corner	120	4500	540000	Escalera, Lorie Ramos	f	Installment	1785213957	1785213957	\N		\N
dd4df040-fc18-4a2b-9a8a-20f898f50aad	11111111-1111-4111-8111-111111111111	Block 6	2	Corner	100	4500	450000	Gumbason, Anna Jean	f	Installment	1785213957	1785213957	\N		\N
465c5fc3-6b6b-4cd4-8cb8-5d502ad36e5c	11111111-1111-4111-8111-111111111111	Block 3	14	Corner	132	4500	594000	Pascual, Nonievy	f	Installment	1785213957	1785213957	\N		\N
006ded05-3f12-4043-827e-5a904273be7e	11111111-1111-4111-8111-111111111111	Block 5	2	Commercial / Corner	100	4500	450000	Fortich, Daryl Jan	f	Installment	1785213957	1785213957	\N		\N
6af44ecc-d70c-4685-ad5d-1f0c34535935	11111111-1111-4111-8111-111111111111	Block 2	6	Commercial	106	4500	477000	Calzada, Monaliza	f	Installment	1785213957	1785213957	\N		\N
39ca0429-da8d-41de-ac9d-1b273a24790f	11111111-1111-4111-8111-111111111111	Block 2	8	Commercial	105	4500	472500	Rosalita, Loviena Ann Garcia	f	Installment	1785213957	1785213957	\N		\N
9ed5c9f9-a39d-4dd8-a23a-0edd2ff30fec	11111111-1111-4111-8111-111111111111	Block 1	7	Inner	100	4300	430000	Bermillo, Jorgeanne Barbarona	f	Installment	1785213957	1785213957	\N		\N
7aec8a04-a575-49b8-8f5e-f3f814465a22	11111111-1111-4111-8111-111111111111	Block 2	1	Commercial / Corner	112	4500	504000	Camallere, Catherine	f	Installment	1785213957	1785213957	\N		\N
c564c6b1-a6b7-49b9-b969-aa35bea842d7	11111111-1111-4111-8111-111111111111	Block 2	5	Inner	100	4300	430000	Ibacarra, Ofelia	f	Installment	1785213957	1785213957	\N		\N
9e5e662f-2626-4387-95d7-44417dae8712	11111111-1111-4111-8111-111111111111	Block 1	17	Commercial / Corner	100	4500	450000	Pastor, Noralyn Suerto	f	Installment	1785213957	1785213957	\N		\N
30ef46da-f2ba-48da-b43e-5828f7a9ad09	11111111-1111-4111-8111-111111111111	Block 3	15	Corner	110	4500	495000	Adiao, Norilyn	f	Installment	1785213957	1785213957	\N		\N
5a465889-4d6d-4840-b0c4-cb91f7ff4527	11111111-1111-4111-8111-111111111111	Block 1	12	Commercial	116	4500	522000	Marimon, Michael	f	Installment	1785213957	1785213957	\N		\N
b66e8e0d-dedc-4d8a-aa45-6e9f499c7d17	11111111-1111-4111-8111-111111111111	Block 1	8	Commercial	114	4500	513000	Amor, Lexcelle	f	Installment	1785213957	1785213957	\N		\N
\.


--
-- Data for Name: contracts; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."contracts" ("id", "project_id", "lot_id", "buyer_name", "buyer_address", "buyer_gmail", "buyer_contact", "lot_block", "lot_lot", "lot_area", "lot_type", "lot_rate", "contract_price", "payment_plan", "initial_payment", "term_years", "monthly_amortization", "due_day", "next_due_date", "approval_at", "marketing_representative", "agent_code", "selling_agent_id", "source_of_buyer", "other_source", "particulars", "created_at", "updated_at", "agent_id", "broker_id", "buyer_last_name", "buyer_first_name", "buyer_middle_name", "term_months", "agent_commission_split_months", "buyer_user_id", "is_promo", "list_price", "amort_start_date", "penalty_waived_through_due_date", "first_installment_amount") FROM stdin;
9f4b150a-ee3a-45f0-a19e-b50580f2bf71	11111111-1111-4111-8111-111111111111	b66e8e0d-dedc-4d8a-aa45-6e9f499c7d17	Amor, Lexcelle	 		09561325339	Block 1	8	114	Commercial	4500	513000	installment	0	7	6107.142857142857	10	2026-09-10	2026-06-21		THS-006002	8d058edb-98b1-4321-83b9-99fa79cd901d	{}		Regular payment - August 16, 2026	1786241397	1786947055	d4308329-b467-4ebf-a828-e463b491656e	\N	Amor	Lexcelle		84	30	53e14aaf-6ef2-428f-a506-17309494c9fa	f	513000	\N	2026-08-10	\N
57f0bcb4-ba3b-4dc1-8033-650f171af16d	11111111-1111-4111-8111-111111111111	2241a991-8dd8-4195-806a-9be204df1b31	Algones, Ejea	 	ejeaalgones1219@gmai.com	09622630570	Block 5	4	100	Inner	4300	430000	installment	0	7	5119.047619047619	29	2026-08-29	2026-06-28		THS-006002	8d058edb-98b1-4321-83b9-99fa79cd901d	{"Agent Endorsed"}		Regular payment - August 28, 2026	1786246188	1787879083	8d058edb-98b1-4321-83b9-99fa79cd901d	\N	Algones	Ejea		84	30	787c8487-9a54-411e-abc2-174cf4fa7c09	f	430000	\N	\N	\N
7549101b-ff5c-48d4-9733-e91741b01c13	11111111-1111-4111-8111-111111111111	30ef46da-f2ba-48da-b43e-5828f7a9ad09	Adiao, Norilyn		adiaonorilyn@gmail.com		Block 3	15	110	Corner	4500	495000	installment	0	7	5892.857142857143	5	2026-09-05	2026-06-21			d4308329-b467-4ebf-a828-e463b491656e	{}		Regular payment - August 8, 2026	1786245673	1788072813	d4308329-b467-4ebf-a828-e463b491656e	\N	Adiao	Norilyn		84	30	81ce8e27-1745-4ed9-abe4-65bbc8ba8012	f	495000	2026-06-05	2026-07-05	\N
67331e2f-06f3-4043-a6b7-b9694d307e32	11111111-1111-4111-8111-111111111111	eb7ba35a-ea74-4636-89ab-cd6c8c5f27bf	Suerto, Gemma				Block 1	16	157	Commercial / Corner	4500	300000	half	150000	0	25000	30	2026-08-30	2026-06-30		THS-006002	8d058edb-98b1-4321-83b9-99fa79cd901d	{"Agent Endorsed"}		Regular payment - July 30, 2026	1786250249	1786250309	8d058edb-98b1-4321-83b9-99fa79cd901d	\N	Suerto	Gemma		6	6	bdf8af47-610b-4d69-b86f-29415b703d7d	t	706500	\N	\N	\N
497dd3c9-419b-4169-9614-8fc149df2174	11111111-1111-4111-8111-111111111111	1703885a-39d9-4158-81e6-2574bfe8a6c1	Dongiapon, Jenilyn Pagapong		jenilyndongiapon@gmail.com	09541672174	Block 2	16	100	Commercial / Corner	4500	450000	installment	0	7	5357.142857142857	30	2026-08-30	2026-07-12		THS-006002	8d058edb-98b1-4321-83b9-99fa79cd901d	{"Agent Endorsed"}		First monthly payment - July 12, 2026	1786249873	1786947057	8d058edb-98b1-4321-83b9-99fa79cd901d	\N	Dongiapon	Jenilyn	Pagapong	84	30	02041dc4-d46a-4c26-b973-f3880ce58b8e	f	450000	\N	\N	\N
f5cb0caa-c0e8-4032-93a1-f0415fbf626c	11111111-1111-4111-8111-111111111111	dd4df040-fc18-4a2b-9a8a-20f898f50aad	Gumbason, Anna Jean		annajeangumbason121022@gmail.com	09271505792	Block 6	2	100	Corner	4500	450000	installment	0	7	5357.142857142857	30	2026-09-30	2026-07-12		THS-006002	8d058edb-98b1-4321-83b9-99fa79cd901d	{"Agent Endorsed"}		Regular payment - August 27, 2026	1786249978	1787815388	8d058edb-98b1-4321-83b9-99fa79cd901d	\N	Gumbason	Anna Jean		84	30	5bcc9fcc-0cca-44da-834a-20a307511776	f	450000	\N	\N	\N
157fa186-4862-4260-a0c0-3a06fba6cc58	11111111-1111-4111-8111-111111111111	981e0794-a981-42e2-a2a1-93706e6f3a50	Escalera, Lorie Ramos	 	zgsorian12@gmail.com	09276084952	Block 4	1	120	Corner	4500	540000	installment	0	7	6428.571428571428	15	2026-08-15	2026-06-21		THS-006003	7aca5aa5-3c18-4791-9cfc-3487540f0f74	{}		Multi-lot payment - July 20, 2026	1786242595	1787101273	d4308329-b467-4ebf-a828-e463b491656e	\N	Escalera	Lorie	Ramos	84	30	3353fad4-28dc-4432-a090-2a515bf46f8c	f	540000	\N	2026-08-15	\N
ca0814b1-4e68-420a-a9c8-d3a5439e5f8c	11111111-1111-4111-8111-111111111111	9ed5c9f9-a39d-4dd8-a23a-0edd2ff30fec	Bermillo, Jorgeanne Barbarona		jabermillo.02@gmail.com	09297045463	Block 1	7	100	Inner	4300	430000	installment	0	7	5119.047619047619	30	2026-08-30	2026-07-20		THS-006005	33b1d7b8-ab93-413b-86c7-8622cda2e636	{"Agent Endorsed"}		First monthly payment - July 20, 2026	1786420486	1786947055	33b1d7b8-ab93-413b-86c7-8622cda2e636	\N	Bermillo	Jorgeanne	Barbarona	84	30	8fcbfd05-41bc-4258-90d6-9ac255939824	f	430000	\N	\N	\N
5ddd4e14-b66e-435a-8421-82bf1db8d623	11111111-1111-4111-8111-111111111111	f060546c-f9ce-41f8-a6ba-d7ef2a0228e2	Escalera, Lorie Ramos	 	zgsorian12@gmail.com	09276084952	Block 4	2	100	Corner	4500	450000	installment	0	7	5357.142857142857	15	2026-08-15	2026-06-21		THS-006003	7aca5aa5-3c18-4791-9cfc-3487540f0f74	{}		Multi-lot payment - July 20, 2026	1786242597	1787101274	d4308329-b467-4ebf-a828-e463b491656e	\N	Escalera	Lorie	Ramos	84	30	3353fad4-28dc-4432-a090-2a515bf46f8c	f	450000	\N	2026-08-15	\N
285ad740-cab8-481b-b77d-3f0e52700de9	11111111-1111-4111-8111-111111111111	7aec8a04-a575-49b8-8f5e-f3f814465a22	Camallere, Catherine		catherinecamallere51@gmail.com	09056564890	Block 2	1	112	Commercial / Corner	4500	504000	installment	0	3	14000	15	2026-07-15	2026-06-15			d4308329-b467-4ebf-a828-e463b491656e	{"Agent Endorsed"}		First monthly payment - June 15, 2026	1786253907	1786947057	d4308329-b467-4ebf-a828-e463b491656e	\N	Camallere	Catherine		36	30	3ec1cee0-0c96-4345-8c5e-93f2c4e726e7	f	504000	\N	2026-07-15	\N
88cf7126-51a1-4337-9fa8-7a4748f19c32	11111111-1111-4111-8111-111111111111	cdc781d5-049c-4998-8db6-a4f2ef3da162	Cacho, Lourdes Quinoo		cachlour@gmail.com	09505387800	Block 1	10	115	Commercial	4500	517500	installment	0	7	6160.714285714285	18	2026-08-18	2026-06-09		THS-006001	0edee6f7-5ab2-41b1-b754-3ebba9a2d0b5	{}		Regular payment - August 9, 2026	1786240055	1787458293	0edee6f7-5ab2-41b1-b754-3ebba9a2d0b5	\N	Cacho	Lourdes	Quinoo	84	30	58337c4b-72ba-4229-bc30-e9f90f6efc14	f	517500	\N	2026-08-18	\N
9ab5350a-319c-44e3-9e78-63ed89b26694	11111111-1111-4111-8111-111111111111	d7279f30-fd32-471e-935f-688ae12d8852	Marces, Erwin		erwinmarces26@gmail.com	09094079123	Block 1	13	100	Inner	4300	200000	installment	0	0	70000	27	2026-09-27	2026-07-27		THS-006005	33b1d7b8-ab93-413b-86c7-8622cda2e636	{"Agent Endorsed"}		Regular payment - August 26, 2026	1786250749	1787727116	33b1d7b8-ab93-413b-86c7-8622cda2e636	\N	Marces	Erwin		3	3	2b71c15a-8dbf-4d49-a887-7a58c35612df	t	430000	\N	\N	60000
4e31e8e2-fdda-491e-b090-706d53c5dac4	11111111-1111-4111-8111-111111111111	6af44ecc-d70c-4685-ad5d-1f0c34535935	Calzada, Monaliza		monadelacalzada@gmail.com	09458129719	Block 2	6	106	Commercial	4500	477000	installment	0	7	5678.571428571428	30	2026-09-30	2026-07-05		THS-006004	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	{"Agent Endorsed"}		Regular payment - August 30, 2026	1786247675	1788073389	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	\N	Calzada	Monaliza		84	30	194a87ae-a361-410b-920d-1f5a2f9eb5ba	f	477000	\N	\N	\N
8b55893e-d779-4629-a80d-171aa9bf2f43	11111111-1111-4111-8111-111111111111	af5a2877-7eef-4fc8-b3a8-63639cfd9a8f	Bermillo, Jorgeanne Barbarona		jabermillo.02@gmail.com	09297045463	Block 1	6	113	Commercial	4500	508500	installment	0	7	6053.571428571428	10	2026-09-10	2026-07-10		THS-006005	33b1d7b8-ab93-413b-86c7-8622cda2e636	{"Agent Endorsed"}		Regular payment - August 10, 2026	1786420919	1786947055	33b1d7b8-ab93-413b-86c7-8622cda2e636	\N	Bermillo	Jorgeanne	Barbarona	84	30	8fcbfd05-41bc-4258-90d6-9ac255939824	f	508500	\N	2026-08-10	\N
b34a9227-633a-4ee4-b45b-e5fc2b9d64e0	11111111-1111-4111-8111-111111111111	5a465889-4d6d-4840-b0c4-cb91f7ff4527	Marimon, Michael		nk.mmarimon@gmail.com	09506912666	Block 1	12	116	Commercial	4500	522000	installment	0	7	6214.285714285715	18	2026-09-18	2026-06-09			d4308329-b467-4ebf-a828-e463b491656e	{"Broker Endorsed"}		Regular payment - August 14, 2026	1786240934	1786947061	d4308329-b467-4ebf-a828-e463b491656e	\N	Marimon	Michael		84	30	dfbb3ed5-8623-4f80-a728-c96de6eb77ea	f	522000	\N	\N	\N
f548b2a2-2dd6-447f-a1a2-6016a3797417	11111111-1111-4111-8111-111111111111	c0f12af6-6494-47ff-8809-24beea8a6a1e	Marolina, Zaryll Praise		zeepraise.maro@gmail.com	09664623124	Block 5	1	119	Corner	4500	535500	installment	0	7	6375	7	2026-09-07	2026-07-05		THS-006004	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	{"Agent Endorsed"}		Regular payment - August 10, 2026	1786247400	1786947061	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	\N	Marolina	Zaryll Praise		84	30	604af0a9-ea54-44df-9af0-4c8e87705753	f	535500	\N	2026-08-07	\N
8dfbcc8e-70ec-41cb-b245-9a7ef481618e	11111111-1111-4111-8111-111111111111	02f24272-106f-4126-83c6-a438ed9c128f	Soriano, Julius Aldemita	 	zgsoriano12@gmail.com	09667583677	Block 5	6	100	Inner	4300	430000	installment	0	7	5119.047619047619	30	2026-08-30	2026-07-12		THS-006005	33b1d7b8-ab93-413b-86c7-8622cda2e636	{"Agent Endorsed"}		First monthly payment - August 9, 2026	1786249288	1786947061	33b1d7b8-ab93-413b-86c7-8622cda2e636	\N	Soriano	Julius	Aldemita	84	30	904c9ced-a24b-4887-8d6e-01feda354383	f	430000	\N	\N	\N
0e777ef1-8077-449b-bcd0-0450d883863a	11111111-1111-4111-8111-111111111111	465c5fc3-6b6b-4cd4-8cb8-5d502ad36e5c	Pascual, Nonievy	 	nonievy11@gmail.com		Block 3	14	132	Corner	4500	594000	installment	0	7	7071.428571428572	30	2026-09-30	2026-06-21		THS-0021	d4308329-b467-4ebf-a828-e463b491656e	{}		Regular payment - August 27, 2026	1786242876	1787828928	d4308329-b467-4ebf-a828-e463b491656e	\N	Pascual	Nonievy		84	30	508ad9fd-c6d7-4ff6-b09c-ccf0cde4d6d8	f	594000	\N	\N	\N
33e8f5c0-8bdd-46de-8591-598f1a7c835d	11111111-1111-4111-8111-111111111111	3f86fb37-d203-49e2-92fd-8585e8e53bc7	Lumingkit, Ariel Arnais		ariellumingkit13@gmail.com	09155611342	Block 6	1	116	Corner	4500	522000	installment	0	7	6214.285714285715	30	2026-09-30	2026-06-30			d4308329-b467-4ebf-a828-e463b491656e	{"Agent Endorsed"}		Regular payment - August 28, 2026	1786246508	1787888859	d4308329-b467-4ebf-a828-e463b491656e	\N	Lumingkit	Ariel	Arnais	84	30	0bbd5e2b-9966-4c99-a153-17e93ac8b8ae	f	522000	\N	\N	\N
eef7f87c-1988-4c3a-9436-1af1d58e0c00	11111111-1111-4111-8111-111111111111	c564c6b1-a6b7-49b9-b969-aa35bea842d7	Ibacarra, Ofelia		ofeliabacarra@gmail.com	09499289550	Block 2	5	100	Inner	4300	430000	installment	0	6	5972.222222222223	30	2026-09-30	2026-08-01		THS-006004	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	{"Agent Endorsed"}		Regular payment - August 1, 2026	1786252255	1786947059	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	\N	Ibacarra	Ofelia		72	30	4f7d4ac8-9d67-40d8-9f44-23bdcc0ad903	f	430000	\N	2026-08-01	\N
550472b0-f8d9-4c72-932a-e74906575d78	11111111-1111-4111-8111-111111111111	9e5e662f-2626-4387-95d7-44417dae8712	Pastor, Noralyn Suerto	 	xierasue@gmail.com	09264205472	Block 1	17	100	Commercial / Corner	4500	450000	installment	0	7	5357.142857142857	6	2026-09-06	2026-06-21		THS-006002	8d058edb-98b1-4321-83b9-99fa79cd901d	{}		Regular payment - August 4, 2026	1786242104	1786947059	d4308329-b467-4ebf-a828-e463b491656e	\N	Pastor	Noralyn	Suerto	84	30	48339372-3066-4168-9658-0dd812512c5b	f	450000	\N	2026-08-06	\N
c274de29-923d-4aa1-b55e-4b7eb752e919	11111111-1111-4111-8111-111111111111	39ca0429-da8d-41de-ac9d-1b273a24790f	Rosalita, Loviena Ann Garcia			447432530539	Block 2	8	105	Commercial	4500	472500	installment	0	5	7875	5	2026-10-05	2026-07-10		THS-006004	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	{"Agent Endorsed"}		Regular payment - August 31, 2026	1786328485	1788171519	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	\N	Rosalita	Loviena Ann	Garcia	60	30	36e9e804-ed7a-45c6-bcf1-87daa36e74ca	f	472500	\N	2026-08-05	\N
c243f864-cc19-4486-b2b5-b3a50f3d8501	11111111-1111-4111-8111-111111111111	4911c421-5871-41d5-9b4d-75feed3d479b	Florida, Wilmie	 	wilmieflorida@icloud.com	09062568380	Block 1	14	117	Commercial	4500	526500	installment	0	7	6267.857142857143	5	2026-08-05	2026-06-09			d4308329-b467-4ebf-a828-e463b491656e	{}		Regular payment - August 9, 2026	1786240434	1786947059	d4308329-b467-4ebf-a828-e463b491656e	\N	Florida	Wilmie		84	30	d1206ad9-f01b-48e8-ac00-b7f4bcf4b63a	f	526500	\N	2026-08-05	\N
bdaff66c-10b6-4b37-a4dd-f10c58be5e8c	11111111-1111-4111-8111-111111111111	2241a991-8dd8-4195-806a-9be204df1b31	Soriano, Julius Aldemita	 	zgsoriano12@gmail.com	09667583677	Block 5	4	100	Inner	4300	430000	installment	0	7	5119.047619047619	30	2026-08-30	2026-07-12		THS-006005	33b1d7b8-ab93-413b-86c7-8622cda2e636	{"Agent Endorsed"}		First monthly payment - August 9, 2026	1786249291	1786947062	33b1d7b8-ab93-413b-86c7-8622cda2e636	\N	Soriano	Julius	Aldemita	84	30	904c9ced-a24b-4887-8d6e-01feda354383	f	430000	\N	\N	\N
b796cce8-d6b0-45cb-8587-fdd5811ad8c6	11111111-1111-4111-8111-111111111111	86663e2f-a8cd-4864-95b2-5055457a9ee8	Escalera, Lorie Ramos	 	zgsorian12@gmail.com	09276084952	Block 4	3	100	Inner	4300	430000	installment	0	7	5119.047619047619	15	2026-08-15	2026-06-21		THS-006003	7aca5aa5-3c18-4791-9cfc-3487540f0f74	{}		Multi-lot payment - July 20, 2026	1786242599	1787101275	d4308329-b467-4ebf-a828-e463b491656e	\N	Escalera	Lorie	Ramos	84	30	3353fad4-28dc-4432-a090-2a515bf46f8c	f	430000	\N	2026-08-15	\N
931d29ee-5fa4-4cf4-b381-79de39ccf5f5	11111111-1111-4111-8111-111111111111	006ded05-3f12-4043-827e-5a904273be7e	Fortich, Daryl Jan		deefouur@gmail.com	09163757099	Block 5	2	100	Commercial / Corner	4500	450000	installment	0	7	5357.142857142857	30	2026-09-30	2026-07-05		THS-006004	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	{"Agent Endorsed"}		Regular payment - August 30, 2026	1786247263	1788056670	bdba8dbd-cbe5-42bb-bb63-534317fff9b0	\N	Fortich	Daryl Jan		84	30	9a81fb0c-f487-4b12-9102-8aa6ea22ecff	f	450000	\N	\N	\N
\.


--
-- Data for Name: contract_split_history; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."contract_split_history" ("id", "contract_id", "split_months", "effective_period_start", "created_at", "strategy", "rebalance_strategy", "late_payment_split_mode") FROM stdin;
4947ceec-525e-40f0-9e1a-e70bb0d4d37f	9f4b150a-ee3a-45f0-a19e-b50580f2bf71	20	2026-06-16	1786421157	catch_up	catch_up	keep_due_period_split
e425ca2e-d1fd-48fd-b4f5-2d45da957b8c	57f0bcb4-ba3b-4dc1-8033-650f171af16d	20	2026-06-16	1786421157	catch_up	catch_up	keep_due_period_split
24b712b0-55e6-4e9f-aa07-910f216396d8	7549101b-ff5c-48d4-9733-e91741b01c13	20	2026-06-16	1786421157	catch_up	catch_up	keep_due_period_split
8722087f-8ca0-4d53-9945-2d25433feec2	8b55893e-d779-4629-a80d-171aa9bf2f43	20	2026-07-01	1786421158	catch_up	catch_up	keep_due_period_split
3942e112-008f-44e2-a985-e291366b8f48	88cf7126-51a1-4337-9fa8-7a4748f19c32	20	2026-06-01	1786421158	catch_up	catch_up	keep_due_period_split
7b204740-1b39-4ca0-9c39-8e260167fed0	ca0814b1-4e68-420a-a9c8-d3a5439e5f8c	20	2026-07-16	1786421158	catch_up	catch_up	keep_due_period_split
7167661e-ad94-44bd-9a82-1223bb3413bb	4e31e8e2-fdda-491e-b090-706d53c5dac4	20	2026-07-01	1786421160	catch_up	catch_up	keep_due_period_split
9b1abb04-ef83-4efe-9165-e8d6832483e8	285ad740-cab8-481b-b77d-3f0e52700de9	15	2026-06-01	1786421160	catch_up	catch_up	keep_due_period_split
634af3ab-10e1-41f7-9a08-3383f8274872	497dd3c9-419b-4169-9614-8fc149df2174	20	2026-07-01	1786421160	catch_up	catch_up	keep_due_period_split
fb05f7a2-e023-42a8-9b4e-daffed7d9953	b796cce8-d6b0-45cb-8587-fdd5811ad8c6	20	2026-06-16	1786421160	catch_up	catch_up	keep_due_period_split
11622377-2602-4879-9140-5b8db3119885	5ddd4e14-b66e-435a-8421-82bf1db8d623	20	2026-06-16	1786421160	catch_up	catch_up	keep_due_period_split
15209846-a60e-480f-9421-c04071967bc6	157fa186-4862-4260-a0c0-3a06fba6cc58	20	2026-06-16	1786421160	catch_up	catch_up	keep_due_period_split
a6666dd9-7151-48b6-8280-8fd5b2f67daa	c243f864-cc19-4486-b2b5-b3a50f3d8501	20	2026-06-01	1786421162	catch_up	catch_up	keep_due_period_split
80ef2f8c-6cee-4d26-8c0f-c2a446fed5d2	931d29ee-5fa4-4cf4-b381-79de39ccf5f5	20	2026-07-01	1786421162	catch_up	catch_up	keep_due_period_split
08faa825-cbf8-4450-953e-c8177fc1b949	eef7f87c-1988-4c3a-9436-1af1d58e0c00	20	2026-07-16	1786946879	catch_up	catch_up	keep_due_period_split
3541cc02-4b59-4697-b041-1b252b2ac16b	f5cb0caa-c0e8-4032-93a1-f0415fbf626c	20	2026-07-01	1786421162	catch_up	catch_up	keep_due_period_split
f3ec84fe-dc36-48fd-9f31-5bd3ab41035e	33e8f5c0-8bdd-46de-8591-598f1a7c835d	20	2026-06-16	1786421162	catch_up	catch_up	keep_due_period_split
35979af9-26d5-411c-8377-d99a290dee79	b34a9227-633a-4ee4-b45b-e5fc2b9d64e0	20	2026-06-01	1786421162	catch_up	catch_up	keep_due_period_split
4679134f-3274-4766-8634-1c27f0acb797	f548b2a2-2dd6-447f-a1a2-6016a3797417	20	2026-07-01	1786421162	catch_up	catch_up	keep_due_period_split
b8074a2e-fb2a-48a4-8cdd-94e9a8fd71bc	0e777ef1-8077-449b-bcd0-0450d883863a	20	2026-06-16	1786421164	catch_up	catch_up	keep_due_period_split
f0af90cc-1356-415c-b616-9946b1c560ac	550472b0-f8d9-4c72-932a-e74906575d78	20	2026-06-16	1786421164	catch_up	catch_up	keep_due_period_split
d7e66a8f-9046-49a0-8185-dc4250186f1c	8dfbcc8e-70ec-41cb-b245-9a7ef481618e	20	2026-07-01	1786421164	catch_up	catch_up	keep_due_period_split
70ebbbd2-a952-4dad-9884-d46b34f95326	c274de29-923d-4aa1-b55e-4b7eb752e919	15	2026-07-01	1786421164	catch_up	catch_up	keep_due_period_split
c9f2c387-22bc-45e1-b558-80b64c3d5508	bdaff66c-10b6-4b37-a4dd-f10c58be5e8c	20	2026-07-01	1786421164	catch_up	catch_up	keep_due_period_split
8780740d-dd4d-48bc-86cc-2bf7921f2806	57f0bcb4-ba3b-4dc1-8033-650f171af16d	30	2026-08-01	1786947055	catch_up	even_split	keep_due_period_split
adb25fd3-2b20-4c66-b5e3-90d0ebc1b065	9f4b150a-ee3a-45f0-a19e-b50580f2bf71	30	2026-08-01	1786947055	catch_up	even_split	keep_due_period_split
1d6fa482-fdf0-4d10-a5ee-c803852c02a5	8b55893e-d779-4629-a80d-171aa9bf2f43	30	2026-08-01	1786947055	catch_up	even_split	keep_due_period_split
ea02611f-914f-4c1d-b172-3f19fd255579	ca0814b1-4e68-420a-a9c8-d3a5439e5f8c	30	2026-08-01	1786947055	catch_up	even_split	keep_due_period_split
a9e2da7d-3d64-4415-8951-12bae3b268c9	88cf7126-51a1-4337-9fa8-7a4748f19c32	30	2026-08-01	1786947055	catch_up	even_split	keep_due_period_split
9c6ad198-0791-4a0f-98c0-6d56201ab569	4e31e8e2-fdda-491e-b090-706d53c5dac4	30	2026-08-01	1786947057	catch_up	even_split	keep_due_period_split
35498b47-1e39-43b5-9407-b66f730b6354	285ad740-cab8-481b-b77d-3f0e52700de9	30	2026-08-01	1786947057	catch_up	even_split	keep_due_period_split
1319b681-fb55-4947-b747-645007bdac5e	497dd3c9-419b-4169-9614-8fc149df2174	30	2026-08-01	1786947057	catch_up	even_split	keep_due_period_split
3091a9e9-640d-4187-a8cb-9bdfd5db26d2	157fa186-4862-4260-a0c0-3a06fba6cc58	30	2026-08-01	1786947057	catch_up	even_split	keep_due_period_split
443ec60e-8495-4cb3-8d55-3387df01fe22	b796cce8-d6b0-45cb-8587-fdd5811ad8c6	30	2026-08-01	1786947057	catch_up	even_split	keep_due_period_split
fc995831-0dc3-402d-b170-05477ac1f7f9	5ddd4e14-b66e-435a-8421-82bf1db8d623	30	2026-08-01	1786947057	catch_up	even_split	keep_due_period_split
32aeac72-41b9-47d9-860e-b60005124b7e	c243f864-cc19-4486-b2b5-b3a50f3d8501	30	2026-08-01	1786947059	catch_up	even_split	keep_due_period_split
b0ead255-964a-4a53-a38a-3fcee1fe57c5	931d29ee-5fa4-4cf4-b381-79de39ccf5f5	30	2026-08-01	1786947059	catch_up	even_split	keep_due_period_split
0033b38a-68ef-4bce-a705-db6dd3faafd9	f5cb0caa-c0e8-4032-93a1-f0415fbf626c	30	2026-08-01	1786947059	catch_up	even_split	keep_due_period_split
8797269f-9b14-43e1-a09d-34322f4e4501	eef7f87c-1988-4c3a-9436-1af1d58e0c00	30	2026-08-01	1786947059	catch_up	even_split	keep_due_period_split
587dac45-9e29-4091-99cd-fb5cb129f880	33e8f5c0-8bdd-46de-8591-598f1a7c835d	30	2026-08-01	1786947059	catch_up	even_split	keep_due_period_split
c6ddcce5-ed00-437c-9e49-4676f604e7a2	550472b0-f8d9-4c72-932a-e74906575d78	30	2026-08-01	1786947059	catch_up	even_split	keep_due_period_split
ed76f56e-b6f6-4682-becc-c5b457b51671	b34a9227-633a-4ee4-b45b-e5fc2b9d64e0	30	2026-08-01	1786947061	catch_up	even_split	keep_due_period_split
b7fee748-5e32-4420-b5f6-20a70935e191	c274de29-923d-4aa1-b55e-4b7eb752e919	30	2026-08-01	1786947061	catch_up	even_split	keep_due_period_split
7a7d61d3-cbe1-4189-8d41-a2074c381fb8	f548b2a2-2dd6-447f-a1a2-6016a3797417	30	2026-08-01	1786947061	catch_up	even_split	keep_due_period_split
3c18305c-3d0a-412c-bc38-ada21b72168f	0e777ef1-8077-449b-bcd0-0450d883863a	30	2026-08-01	1786947061	catch_up	even_split	keep_due_period_split
829cc4d1-a399-4b70-b199-e52523f9c963	8dfbcc8e-70ec-41cb-b245-9a7ef481618e	30	2026-08-01	1786947061	catch_up	even_split	keep_due_period_split
53fd4f3b-9d47-46d1-8f70-280533e15585	bdaff66c-10b6-4b37-a4dd-f10c58be5e8c	30	2026-08-01	1786947062	catch_up	even_split	keep_due_period_split
23cb21bb-bce5-4c42-b644-6478ccf4f8f6	7549101b-ff5c-48d4-9733-e91741b01c13	30	2026-08-01	1788072813	catch_up	even_split	adopt_new_split
\.


--
-- Data for Name: employee_position_types; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."employee_position_types" ("id", "label", "sort_order", "created_at", "updated_at") FROM stdin;
38937431-268a-4517-b670-86970dea8bf6	Executive Secretary	0	1786606285	1786606285
6923ab70-8dd2-4196-bd6c-12941f7ca54a	Assistant Secretary	1	1786606440	1786606440
\.


--
-- Data for Name: expense_categories; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."expense_categories" ("id", "name", "created_at") FROM stdin;
2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	Refund Record	1786594636
261468ec-0d01-4773-8dc2-3192345e62a7	Expenses Record	1786594656
2d80e5dc-6956-40a0-af0a-fb4fa87d9e9d	Bills	1786949265
\.


--
-- Data for Name: expenses; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."expenses" ("id", "category_id", "paid_to", "description", "amount", "paid_at", "created_at") FROM stdin;
7bce471a-21cf-4aca-91e0-72e4d7eecdb8	2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	Ficco	Partial Payment 100k	100000	2026-06-30	1786595143
854bd09b-80ee-4ddd-9a96-6f5c307b6262	2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	DiamonDray B Villamor	Office Furniture & Fixtures Transfer	1500	2026-06-30	1786595316
9cc6a4a8-e416-43b7-a0c9-3b3213d5f461	2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	Dodong Electrician / c/o Annafe Castillo	Electrician Office Transfer Fee	1550	2026-06-29	1786596151
d6329d29-9ad5-4807-9742-bab9dcb13b18	2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	DiamonDray B. Villamor	2nd Office Furniture & Fixtures Transfer Fee	1500	2026-07-08	1786596266
6f1c8bfd-986d-4dd9-b0b0-eea34c3f419a	2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	RBMC Alrem Services	Office Aircon Transfer Fees	8500	2026-07-06	1786596335
b8c2fe25-fc80-47cf-8308-8acad4276f4e	2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	DV DEVT. Corporation	Office Space Rental	53492.94	2026-07-02	1786596415
4439a438-2c8b-40dc-9b5d-3d389a500634	261468ec-0d01-4773-8dc2-3192345e62a7	SM Store	Office Supply	1185	2026-07-17	1786596711
6c59139a-47e0-46a1-a809-b19767cf74e3	261468ec-0d01-4773-8dc2-3192345e62a7	HITECH IT SOLUTIONS	Refill INK	1000	2026-07-14	1786597007
f6e7b2a4-5768-4ba4-86c3-ef6243b9ea50	261468ec-0d01-4773-8dc2-3192345e62a7	Secretary	Load	300	2026-07-29	1786597213
cdd3c0b7-e9fe-4605-846b-e6061f8d290e	261468ec-0d01-4773-8dc2-3192345e62a7	Secretary	Load	300	2026-07-16	1786597244
020fc324-f426-4370-836a-6b0c92ebdc14	261468ec-0d01-4773-8dc2-3192345e62a7	Mia Maison	White Tea and Ginger	450	2026-07-16	1786597340
afe773c3-796c-48df-87b6-47af49939ed7	261468ec-0d01-4773-8dc2-3192345e62a7	Petron	Company Vehicle / Meeting, Tripping Gas	1000	2026-07-13	1786597399
870f8747-5338-47da-9589-9e028eff0336	261468ec-0d01-4773-8dc2-3192345e62a7	Papercot	Company Docs Photocopy	37	2026-07-28	1786597467
71a3f8d4-0b75-430a-a8cf-47cc84a1f9c1	261468ec-0d01-4773-8dc2-3192345e62a7	Petron Services Station	Company vehicle / Meeting, Tripping Gas	1500	2026-07-09	1786597571
cccb3a6a-b40d-4629-9dbe-dd0e35b1a84a	261468ec-0d01-4773-8dc2-3192345e62a7	Petron	Company Vehicle / Meeting, Tripping Gas	1000	2026-07-26	1786597621
504d3b7a-395f-4ac4-a34c-e60f727b4653	261468ec-0d01-4773-8dc2-3192345e62a7	Petron	Company Vehicle / Meeting, Tripping Gas	1000	2026-07-27	1786597695
75782636-e978-4b91-a68a-bb52ff99a350	261468ec-0d01-4773-8dc2-3192345e62a7	Yucca Bakery  Cafe	Company Meeting Meal	808.5	2026-07-22	1786597791
687b7a07-4a23-471e-a7cc-5605beb78394	261468ec-0d01-4773-8dc2-3192345e62a7	Hagtags Printing Services	Company Uniform Partial Payment	2000	2026-07-30	1786597859
09213068-5684-43b7-9c2d-443d2775781c	261468ec-0d01-4773-8dc2-3192345e62a7	Savemore	Office Supply	170	2026-07-30	1786597906
9a1f2134-ef6d-4148-a715-adb324fddd98	261468ec-0d01-4773-8dc2-3192345e62a7	Savemore	Company Snacks	225	2026-07-30	1786597998
1fb799f0-0f02-4ec6-a0a7-1f2064a7df61	261468ec-0d01-4773-8dc2-3192345e62a7	Gaisano Pantry Supply	Office Pantry Supply	1124.9	2026-07-22	1786598042
3b3f81ed-2067-44ab-ab9b-d4105b0d367c	261468ec-0d01-4773-8dc2-3192345e62a7	Qashier	Company Snacks	205	2026-07-16	1786598096
8604914e-5db5-4ebb-86b2-89e9c28f5757	261468ec-0d01-4773-8dc2-3192345e62a7	Watsons	Tissue and Paper Bag	140.3	2026-07-16	1786598153
50fefd5f-be9d-4cfa-b64f-07b43a0a96d1	261468ec-0d01-4773-8dc2-3192345e62a7	Beard Papa's	Meeting Snacks	135	2026-07-08	1786598184
e7794c26-bdde-4030-a0fd-abe047ec0007	261468ec-0d01-4773-8dc2-3192345e62a7	Gmall Grocery	Pantry Office Supply	1397.75	2026-07-09	1786598225
7bb13ea0-b9f3-4908-a68b-21b0b9e2f7c3	261468ec-0d01-4773-8dc2-3192345e62a7	ACE Hardware	MicroFiber, Glass cleaner, Creston, Extension cord	1375.1	2026-07-16	1786598395
c2609fc0-7e81-4d21-a117-deea95de3bfb	261468ec-0d01-4773-8dc2-3192345e62a7	Cinnabon	Meeting Snacks	510	2026-07-08	1786598507
7b774e45-e27d-4aef-9e32-f7ee81eeca03	261468ec-0d01-4773-8dc2-3192345e62a7	Rosa Balani	Tripping gas	500	2026-07-08	1786598609
d2b2a323-4efa-4932-ad7a-07f29670fa8f	261468ec-0d01-4773-8dc2-3192345e62a7	Gladez Mendoza	Tripping Gas	500	2026-06-20	1786599458
4b5e3d1e-642b-4975-ab20-b7b8dd469347	261468ec-0d01-4773-8dc2-3192345e62a7	Gladez Mendoza	Tripping Gas	500	2026-06-29	1786599495
3028ef20-c9a1-4d5a-98f4-9dbbb25509b5	2d80e5dc-6956-40a0-af0a-fb4fa87d9e9d	Davao Light	DV Development Corporation	1392	2026-08-17	1786949343
4755c67b-a8df-4d52-b5f5-804965765946	2d80e5dc-6956-40a0-af0a-fb4fa87d9e9d	Water District	DV Development Corporation	571	2026-08-17	1786949394
08c16912-19b0-42c0-aebc-b1b1ee0439ea	261468ec-0d01-4773-8dc2-3192345e62a7	Savemore Bangkal	Office Supply	1311.25	2026-08-18	1787210444
0650d4c5-bd5d-4632-9147-ed8c0c5b3a72	261468ec-0d01-4773-8dc2-3192345e62a7	Manong Maintenance Commercial	Tarpulin initial.	1000	2026-08-28	1787908616
9b968d10-9a3d-4d2b-afe5-1d8e97198dba	2a014bc5-3f9e-403c-b7d1-bb7cac9b9074	DV DEVT. Corporation	MOS Security Deposit	40110	2026-07-02	1788074295
\.


--
-- Data for Name: installment_status_overrides; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."installment_status_overrides" ("id", "project_id", "contract_id", "year", "month", "status", "updated_at") FROM stdin;
9cb30922-9cae-4029-aacd-0ae29e38bc4b	11111111-1111-4111-8111-111111111111	c243f864-cc19-4486-b2b5-b3a50f3d8501	2026	7		1786240696
\.


--
-- Data for Name: password_reset_codes; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."password_reset_codes" ("id", "email", "code", "expires_at", "created_at", "failed_attempts") FROM stdin;
\.


--
-- Data for Name: payments; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."payments" ("id", "contract_id", "amount", "method", "months_covered", "paid_at", "created_at", "reference_no", "bank_name", "sender_name", "receiver_name", "mode_label") FROM stdin;
c7a8c2d8-978c-45ed-9840-94fa0077a1a6	88cf7126-51a1-4337-9fa8-7a4748f19c32	6160.71	gcash	0	2026-06-09	1786240055	5041-682-189867		Cacho, Lourdes	Mark Dave Simyunn	
4083d53f-f45c-4a17-af1e-b4763b12c1da	b34a9227-633a-4ee4-b45b-e5fc2b9d64e0	6215	others	0	2026-06-09	1786240934	3041682426472				Gcash && Cash
d78563a2-0f61-4c2c-8843-f67fe08e57a6	c243f864-cc19-4486-b2b5-b3a50f3d8501	6268	bank	1	2026-07-05	1786241151	AP260704114541182019994	CIMB Bank			
9f55b178-c61a-460f-9159-64a06617debe	b34a9227-633a-4ee4-b45b-e5fc2b9d64e0	6215	gcash	1	2026-07-20	1786241092	3043092571823		Marimon, Michael	Mark Dave Simyunn	
44439797-e54c-40ce-9945-acbab1ddb990	88cf7126-51a1-4337-9fa8-7a4748f19c32	6160.71	gcash	1	2026-07-19	1786241036	5043033050410		Cacho, Lourdes	Mark Dave Simyunn	
d0e54e59-86b4-4838-999b-e0dde6fb6fec	7549101b-ff5c-48d4-9733-e91741b01c13	12000	gcash	2	2026-08-08	1786544186	1043730600869		Adiao, Norilyn	Mark Dave Simyunn	
4a25eaeb-3076-4bde-964c-f4f8eb2c6006	9f4b150a-ee3a-45f0-a19e-b50580f2bf71	6107.14	cash	0	2026-06-21	1786241396					
452d3f83-5435-4784-8593-165da85bbf95	285ad740-cab8-481b-b77d-3f0e52700de9	14000	cash	0	2026-06-15	1786253909					
21d877f0-e796-4217-b166-6516c9441a06	c274de29-923d-4aa1-b55e-4b7eb752e919	7875	gcash	0	2026-07-10	1786328486	'4042729649093 & 4042729926041'		Rosalita, Loviena Ann Garcia	Mark Dave Simyunn	
630761ae-710e-455a-82fc-e559f86af16e	550472b0-f8d9-4c72-932a-e74906575d78	5357.14	cash	0	2026-06-21	1786242104					
8062a84a-26eb-4309-976a-ab0758f46e6b	550472b0-f8d9-4c72-932a-e74906575d78	5367.14	bank	1	2026-07-06	1786242211	BN2026070601082737	BDO Pay			
ddb885e5-c459-4179-8f02-0bcca0c72158	157fa186-4862-4260-a0c0-3a06fba6cc58	6428.57	bank	0	2026-06-21	1786242595	1782030603664	BPI			
6f337375-8263-4d37-9bd6-42fa7642ee39	5ddd4e14-b66e-435a-8421-82bf1db8d623	5357.14	bank	0	2026-06-21	1786242597	1782030603664	BPI			
834cc14a-a34c-4c1b-9ea1-2897c119f581	b796cce8-d6b0-45cb-8587-fdd5811ad8c6	5119.05	bank	0	2026-06-21	1786242599	1782030603664	BPI			
c9ed5905-bfbc-41a2-90e5-3e5478322f40	157fa186-4862-4260-a0c0-3a06fba6cc58	6429.42	bank	1	2026-07-20	1786242661	1620110041087	BPI			
cde6ea02-fca9-4045-ba14-07dbe5d353cd	5ddd4e14-b66e-435a-8421-82bf1db8d623	5357.85	bank	1	2026-07-20	1786242662	1620110041087	BPI			
071c9915-3d47-42d2-91bf-6ba0c46aa5df	b796cce8-d6b0-45cb-8587-fdd5811ad8c6	5119.73	bank	1	2026-07-20	1786242663	1620110041087	BPI			
9fe5bafb-e12b-430f-9b1c-bdd863f0c75d	7549101b-ff5c-48d4-9733-e91741b01c13	5893	gcash	0	2026-06-21	1786245673	42089602434		Adiao, Norilyn	Mark Dave Simyunn	
c3607ae8-dc12-4d05-bce2-1c833b2d25be	33e8f5c0-8bdd-46de-8591-598f1a7c835d	6215	gcash	0	2026-06-30	1786246508	3042354205104		Ariel Arnais Lumingkit	Mark Dave Simyunn	
d587d402-3e20-4210-83ba-e49ff7a0a487	0e777ef1-8077-449b-bcd0-0450d883863a	7072	gcash	0	2026-07-25	1786242876	42089602434		Pascual, Nonievy	Mark Dave simyunn	
fd107b7a-7924-4131-9d24-b8016d9f7336	9ab5350a-319c-44e3-9e78-63ed89b26694	50000	bank	1	2026-08-09	1786544247	BN-20260809-39544124	BDO			
1eadb56e-65e2-4dea-9966-05128610bbc0	57f0bcb4-ba3b-4dc1-8033-650f171af16d	5120	cash	0	2026-06-28	1786246188					
5ab464c5-f81c-4401-9ab5-09ec4a47905d	57f0bcb4-ba3b-4dc1-8033-650f171af16d	5150	cash	1	2026-07-28	1786246284					
6a867de4-01c2-4570-82af-f61a8f485ecc	33e8f5c0-8bdd-46de-8591-598f1a7c835d	6215	gcash	1	2026-07-30	1786246574	3043416304889		Lumingkit, Ariel Arnais	Mark Dave Simyunn	
476fe383-a307-4b10-81cd-bea76744a670	931d29ee-5fa4-4cf4-b381-79de39ccf5f5	5358	gcash	0	2026-07-05	1786247263	8042576735159		Fortich, Daryl Jan M.	Mark Dave Simyunn	
78118ee9-04af-476b-95b8-91d075ff4d06	f548b2a2-2dd6-447f-a1a2-6016a3797417	6375	gcash	0	2026-07-05	1786247400	0042576749921		Zaryll Praise Q. Marolina	Mark Dave Simyunn	
52aef861-5ba3-47ef-b42c-6730ecf0f446	4e31e8e2-fdda-491e-b090-706d53c5dac4	5679	gcash	0	2026-07-05	1786247675	4042576800537		Monaliza Dela Calzada	Mark Dave Simyunn	
7f2f4832-11e7-48c7-8d44-c8a0b720a5ad	497dd3c9-419b-4169-9614-8fc149df2174	5358	cash	0	2026-07-12	1786249873					
034796f4-6d92-4c61-8a8a-600770d3821e	f5cb0caa-c0e8-4032-93a1-f0415fbf626c	5358	cash	0	2026-07-12	1786249977					
e4907120-bb01-45bb-894b-75c68b8080f6	67331e2f-06f3-4043-a6b7-b9694d307e32	150000	cash	0	2026-06-30	1786250249					
c258c0a7-5a6e-488f-a81a-0c6210f56c70	67331e2f-06f3-4043-a6b7-b9694d307e32	25000	bank	1	2026-07-30	1786250307	5402257645975	MetroBank			
0a27e1b0-dd7e-401f-9654-8ae2bea7c0cc	c243f864-cc19-4486-b2b5-b3a50f3d8501	6268	bank	0	2026-06-09	1786240434	ENT2026060911181841	BPI			
be093c4d-5a73-4275-96b7-734273036219	0e777ef1-8077-449b-bcd0-0450d883863a	7080	gcash	1	2026-06-21	1786246049	0043256359632		Pascual, Nonievy	Mark Dave Simyunn	
f2233ab6-b305-43b3-b415-d2750d194814	9ab5350a-319c-44e3-9e78-63ed89b26694	60000	bank	0	2026-07-27	1786250749	BN-20260727-24138664 && 0000863579	BDO			
402767b8-8146-41dc-862c-c00016ccfe8b	bdaff66c-10b6-4b37-a4dd-f10c58be5e8c	5125	others	0	2026-07-12	1786249291	1042808058662				Gcash && Cash
b7aa98f0-656e-480f-8337-3b9f6fc14485	8dfbcc8e-70ec-41cb-b245-9a7ef481618e	5125	others	0	2026-07-12	1786249288	1042808058662				Gcash && Cash
9f01cadb-8513-4076-b271-8dc3dfaf2484	ca0814b1-4e68-420a-a9c8-d3a5439e5f8c	5119.05	bank	0	2026-07-20	1786420486	UB883817	Union Bank			
eceba968-ab00-4fa9-b2ac-96b0e751e755	8b55893e-d779-4629-a80d-171aa9bf2f43	6053.57	gcash	0	2026-07-10	1786420918	0042733570815		Bermillo, Jorgeanne Barbarona	Mark Dave Simyunn	
7a1628c3-8628-45f2-84d9-69c5ae2d834c	9f4b150a-ee3a-45f0-a19e-b50580f2bf71	6108	cash	1	2026-07-19	1786434375					
ac65529c-725b-4ea6-b102-46d96783ad11	c274de29-923d-4aa1-b55e-4b7eb752e919	7875	gcash	1	2026-08-01	1786543574	7043484562416		Rosalita, Loviena Ann Garcia	Mark Dave Simyunn	
ff622040-56a6-4ce8-aa2d-820c5498fbfe	eef7f87c-1988-4c3a-9436-1af1d58e0c00	6000	bank	1	2026-08-01	1786543731	2200000004	Securty Bank			
8b0fca4e-eaf3-40b0-a626-32a00dda9da6	550472b0-f8d9-4c72-932a-e74906575d78	5357.13	bank	1	2026-08-04	1786543811	2408155482	Securty Bank			
e82579fc-390a-497e-8366-351bad330009	f548b2a2-2dd6-447f-a1a2-6016a3797417	6375	gcash	1	2026-08-10	1786544342	0043812412240		Marolina, Zaryll Praise	Mark Dave Simyunn	
1ae275fe-1835-4f81-969d-0173ec2fb634	8b55893e-d779-4629-a80d-171aa9bf2f43	6053.57	bank	1	2026-08-10	1786544463	114741	MariBank			
40ca3db2-9e42-4a81-8ba7-88521ea12d42	b34a9227-633a-4ee4-b45b-e5fc2b9d64e0	6215	gcash	1	2026-08-14	1786719267	3043960443485		Marimon, Michael	Mark Dave Simyunn	
fba7ddb4-c517-492d-9ed4-154b8543f36d	9f4b150a-ee3a-45f0-a19e-b50580f2bf71	6108	cash	1	2026-08-16	1786864425					
4dc0477f-9e19-4f47-a604-61880d6fb61c	9ab5350a-319c-44e3-9e78-63ed89b26694	25000	bank	1	2026-08-26	1787727114	BN-2026082255031239	BDO Pay			
660b8c83-eae9-4dc0-88ec-16b3dc9f63d3	f5cb0caa-c0e8-4032-93a1-f0415fbf626c	5358	gcash	1	2026-08-27	1787815389	9044400628942		Gumbason, Anna Jean	Mark Dave Simyunn	
f8e1a18c-5eb0-440f-978c-c48fedaee3d7	0e777ef1-8077-449b-bcd0-0450d883863a	7080	bank	1	2026-08-27	1787828928	2333672797215134294	Taptap send			
a87c2563-816e-41da-9cca-8c3f7f0e8ebd	57f0bcb4-ba3b-4dc1-8033-650f171af16d	3120	gcash	1	2026-08-28	1787879083	4044423028186		Algones, Ejea	Mark Dave Simyunn	
4c90076b-df7e-45e9-8c61-cb7d5cb50b4d	33e8f5c0-8bdd-46de-8591-598f1a7c835d	6215	gcash	1	2026-08-28	1787888859	3044426626533		Lumingkit, Ariel Arnais	Mark Dave Simyunn	
22c8e3a0-2dbe-444d-ac3f-0f0e96c6deee	931d29ee-5fa4-4cf4-b381-79de39ccf5f5	5360	gcash	1	2026-08-30	1788056671	8044457396634		Fortich, Daryl Jan	Mark Dave Simyunn	
f1fbb79f-3916-42dd-9087-95e3bfff5639	4e31e8e2-fdda-491e-b090-706d53c5dac4	5678.57	bank	1	2026-08-30	1788073389	20260830BNORPHMMXXXB000020000456027	BDO Unibank			
4a3635c4-5ae9-4bd9-84a1-06c2476e7c79	c274de29-923d-4aa1-b55e-4b7eb752e919	7875	gcash	1	2026-08-31	1788171519	7044538403416		Rosalita, Loviena Ann Garcia	Mark Dave Simyunn	
\.


--
-- Data for Name: project_rate_categories; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."project_rate_categories" ("id", "project_id", "label", "percent", "is_agent_pool", "sort_order", "created_at", "updated_at") FROM stdin;
932fdd91-6ff6-421f-93b1-fdeef3b3cae8	11111111-1111-4111-8111-111111111111	Hypomone	25	f	2	1786586931	1786586931
3eb536ed-d66e-44eb-841b-fb0b96cb42f6	11111111-1111-4111-8111-111111111111	Land Owner	40	f	1	1786586931	1786586931
6d3e3a64-153f-439e-9b0f-608d8e25e6ec	11111111-1111-4111-8111-111111111111	Legal Counsel	5	f	0	1786586931	1786586931
fa4c5dde-4138-4fee-89f7-1b290e7cb86a	11111111-1111-4111-8111-111111111111	Project Dev & Processing	10	f	3	1786586931	1786586931
7276b053-d1c8-4f51-9238-f3f6a086053f	11111111-1111-4111-8111-111111111111	Agent Commission	20	t	99	1786586931	1786586931
be000c38-37ea-442e-a1b3-20209853cd7f	f7323136-6005-47bb-bd40-289b402f2028	Legal Counsel	5	f	0	1786594499	1786594499
2e47dc3b-081f-43ef-bf5a-0ee023de485a	f7323136-6005-47bb-bd40-289b402f2028	Land Owner	40	f	1	1786594499	1786594499
71e31a22-d09e-4e44-890f-ec42cb1758c5	f7323136-6005-47bb-bd40-289b402f2028	Hypomone	25	f	2	1786594499	1786594499
57da6cb9-9baf-4cac-9a25-b8388355edb9	f7323136-6005-47bb-bd40-289b402f2028	Project Dev & Processing	10	f	3	1786594499	1786594499
f853f4dc-a7e5-48eb-92ea-4dec9259ebed	f7323136-6005-47bb-bd40-289b402f2028	Agent Commission	20	t	99	1786594499	1786594499
\.


--
-- Data for Name: upline_role_types; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."upline_role_types" ("slug", "label", "base_commission_percent", "portal_path", "sort_order", "created_at", "updated_at", "has_baseline", "direct_sale_pool_percent") FROM stdin;
lead-broker	Lead Broker	5	leadbroker	0	1785117822	1787730561	t	12
titling-officer	Titling Officer	3	titlingofficer	1	1785117822	1787755758	t	12
normal-upline	Normal Upline	0	normalupline	2	1787791079	1787791079	f	\N
\.


--
-- Data for Name: project_upline_role_rates; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."project_upline_role_rates" ("project_id", "upline_role_type_slug", "percent", "created_at", "updated_at", "has_baseline", "direct_sale_pool_percent") FROM stdin;
11111111-1111-4111-8111-111111111111	titling-officer	3	1786586931	1787755750	t	12
11111111-1111-4111-8111-111111111111	lead-broker	5	1786586931	1787755785	t	12
f7323136-6005-47bb-bd40-289b402f2028	titling-officer	3	1786594499	1787755977	t	14
f7323136-6005-47bb-bd40-289b402f2028	lead-broker	3	1786594499	1787755978	t	14
11111111-1111-4111-8111-111111111111	normal-upline	0	1787791079	1787791079	f	\N
f7323136-6005-47bb-bd40-289b402f2028	normal-upline	0	1787791079	1787791079	f	\N
\.


--
-- Data for Name: salary_employees; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."salary_employees" ("id", "name", "position", "status", "created_at", "user_id", "rest_days") FROM stdin;
38332f6d-12c1-4691-a234-cf5661ac0d4d	Jhon Rexey	Executive Secretary	Active	1786606470	\N	{}
\.


--
-- Data for Name: salary_plans; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."salary_plans" ("id", "employee_id", "kind", "start_date", "end_date", "training_fee", "monthly_amount", "schedule_type", "created_at") FROM stdin;
d6a0cbf4-74e5-4512-9e86-75d538c9b488	38332f6d-12c1-4691-a234-cf5661ac0d4d	training	2026-06-10	2026-07-10	8000	\N	\N	1786606506
3d77c205-7118-4f32-adbc-f5ed918bf722	38332f6d-12c1-4691-a234-cf5661ac0d4d	regular	2026-07-12	\N	\N	14000	semimonthly	1786609676
\.


--
-- Data for Name: salary_release_entries; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."salary_release_entries" ("id", "project_id", "employee_id", "period_start", "period_end", "amount", "paid_at", "note", "created_at") FROM stdin;
3c3b643e-ddb7-43e3-9966-3de0688edd4c	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-06-10	2026-07-10	1818.2	2026-06-16	\N	1786667517
e42b327d-653f-4a4b-ae47-8c88ee5901ab	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-06-10	2026-07-10	4000	2026-07-12	\N	1786667544
1cf021e9-30f1-4bb9-8211-1807ad6a0607	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-06-10	2026-07-10	2000	2026-07-21	\N	1786667573
6cf65408-be9d-464b-8a3a-84f557c7a8ee	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-06-10	2026-07-10	181.8	2026-07-16	\N	1786667612
158511cd-2f29-4290-816a-df1ffc03d6cd	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-07-01	2026-07-15	1866.67	2026-07-16	\N	1786667612
e4af711a-46f7-4410-a1cc-477765a586f8	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-07-16	2026-07-31	951.53	2026-07-16	\N	1786667613
04ff9c79-2f1f-4b3e-9796-8fc04225cb67	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-07-16	2026-07-31	2000	2026-07-30	\N	1786667637
d7949abe-1525-4e49-8ca2-d4cc0f058fff	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-07-16	2026-07-31	2000	2026-08-08	\N	1786667661
2e108f2f-a4e5-4539-b146-12b71b94650a	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-07-16	2026-07-31	1000	2026-07-31	\N	1786667714
c9792035-78aa-4e07-b40f-746dc2a94eb8	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-07-16	2026-07-31	1048.47	2026-08-17	\N	1786949621
476e1e83-1938-4737-a93b-364f4d5eafaa	11111111-1111-4111-8111-111111111111	38332f6d-12c1-4691-a234-cf5661ac0d4d	2026-08-01	2026-08-15	4951.53	2026-08-17	\N	1786949622
\.


--
-- Data for Name: sessions; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."sessions" ("id", "user_id", "created_at", "expires_at") FROM stdin;
d3678b80-4fa5-403f-8973-3a923a534c22	c0f45227-660a-4a76-b4da-98b19d781064	1788224605	1788311005
\.


--
-- Data for Name: verification_codes; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY "public"."verification_codes" ("id", "username", "email", "password_hash", "code", "expires_at", "created_at", "access_token", "failed_attempts") FROM stdin;
\.


--
-- Data for Name: buckets; Type: TABLE DATA; Schema: storage; Owner: supabase_storage_admin
--

COPY "storage"."buckets" ("id", "name", "owner", "created_at", "updated_at", "public", "avif_autodetection", "file_size_limit", "allowed_mime_types", "owner_id", "type", "versioning_status") FROM stdin;
thessalieh	thessalieh	\N	2026-07-07 02:18:52.737257+00	2026-07-07 02:18:52.737257+00	t	f	\N	\N	\N	STANDARD	DISABLED
\.


--
-- Data for Name: buckets_analytics; Type: TABLE DATA; Schema: storage; Owner: supabase_storage_admin
--

COPY "storage"."buckets_analytics" ("name", "type", "format", "created_at", "updated_at", "id", "deleted_at") FROM stdin;
\.


--
-- Data for Name: buckets_vectors; Type: TABLE DATA; Schema: storage; Owner: supabase_storage_admin
--

COPY "storage"."buckets_vectors" ("id", "type", "created_at", "updated_at") FROM stdin;
\.


--
-- Data for Name: objects; Type: TABLE DATA; Schema: storage; Owner: supabase_storage_admin
--

COPY "storage"."objects" ("id", "bucket_id", "name", "owner", "created_at", "updated_at", "last_accessed_at", "metadata", "version", "owner_id", "user_metadata", "archived_at", "is_delete_marker", "is_versioned") FROM stdin;
\.


--
-- Data for Name: s3_multipart_uploads; Type: TABLE DATA; Schema: storage; Owner: supabase_storage_admin
--

COPY "storage"."s3_multipart_uploads" ("id", "in_progress_size", "upload_signature", "bucket_id", "key", "version", "owner_id", "created_at", "user_metadata", "metadata") FROM stdin;
\.


--
-- Data for Name: s3_multipart_uploads_parts; Type: TABLE DATA; Schema: storage; Owner: supabase_storage_admin
--

COPY "storage"."s3_multipart_uploads_parts" ("id", "upload_id", "size", "part_number", "bucket_id", "key", "etag", "owner_id", "version", "created_at") FROM stdin;
\.


--
-- Data for Name: vector_indexes; Type: TABLE DATA; Schema: storage; Owner: supabase_storage_admin
--

COPY "storage"."vector_indexes" ("id", "name", "bucket_id", "data_type", "dimension", "distance_metric", "metadata_configuration", "created_at", "updated_at") FROM stdin;
\.


--
-- Name: refresh_tokens_id_seq; Type: SEQUENCE SET; Schema: auth; Owner: supabase_auth_admin
--

SELECT pg_catalog.setval('"auth"."refresh_tokens_id_seq"', 1, false);


--
-- PostgreSQL database dump complete
--

-- \unrestrict EowdIaddKETgsvymLIvj2TCBMZf9WCcQnDUQCXvCTP21X61IN4yOVEX55lIyBAa

RESET ALL;

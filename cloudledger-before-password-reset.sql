--
-- PostgreSQL database dump
--

\restrict z3Egd4luGWpzoel2rXpdFMLIrwaXZIh4SOkGa7eHux1rhNWHUzUbe0bJ3Jz8W6d

-- Dumped from database version 18.6
-- Dumped by pg_dump version 18.6

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

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: yume
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO yume;

--
-- Name: sessions; Type: TABLE; Schema: public; Owner: yume
--

CREATE TABLE public.sessions (
    id uuid NOT NULL,
    token_hash text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    user_id uuid NOT NULL
);


ALTER TABLE public.sessions OWNER TO yume;

--
-- Name: users; Type: TABLE; Schema: public; Owner: yume
--

CREATE TABLE public.users (
    id uuid NOT NULL,
    email text NOT NULL,
    password_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.users OWNER TO yume;

--
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: yume
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
1	create users	2026-10-12 22:34:07.782135+05:30	t	\\x91ca1ee024e0422c96d3aaa2e6d930991dbdaac100d9a119dbd1245ac71f5628e121b37a1f7031a8df66c2f54daae4ac	5807167
2	create sessions	2026-10-14 22:56:01.692391+05:30	t	\\xbbe901a2e0630ee25fff322787f95a1b750cdfe0decb490d035b1a01b59735d16ba0fecb45b54c17aca24bf3cb48c7e4	7399360
\.


--
-- Data for Name: sessions; Type: TABLE DATA; Schema: public; Owner: yume
--

COPY public.sessions (id, token_hash, expires_at, created_at, user_id) FROM stdin;
e179dafa-aec0-4c18-b955-ee286c37e5e3	522139ab7665b42bc4a2ff4837e52174eefee01fb10619f284adc09cbf2a74fc	2026-10-22 02:28:20.111823+05:30	2026-10-15 02:28:20.111823+05:30	fc185569-a6a4-4cad-8690-6036b27739e4
53685d34-4501-403c-9382-ee51a4e06005	3f1a48468215cb2644956ad78c0ae26904bb0fe5d9b124c06a686389ab84a796	2026-10-23 00:20:32.787669+05:30	2026-10-16 00:20:32.787669+05:30	f7b27ccc-7e21-4341-8475-12bdbcd809ed
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: yume
--

COPY public.users (id, email, password_hash, created_at) FROM stdin;
f6f224e9-3280-4d21-b09e-58401312f3ff	tessst@example.com	$argon2id$v=19$m=19456,t=2,p=1$ps/ZIyVhJ06/eMvdwuC30g$rUjnrynW8v08fWVQgjwlGMRzySzmkHdg/RWtl4s5A78	2026-10-13 00:31:19.929963+05:30
a6728ae9-d269-4034-9989-0cc155222a06	tessst@examplkje.com	$argon2id$v=19$m=19456,t=2,p=1$KYhc0xplWi2sZkNX2xQ9sQ$y2jnFq7x2MUJhHPObTPSs0i4rFkJdDILGmKBzMiXXeM	2026-10-13 00:34:23.291602+05:30
fc185569-a6a4-4cad-8690-6036b27739e4	you@example.com	$argon2id$v=19$m=19456,t=2,p=1$QrVOjCPl7fqemh9kZSwKSw$QGF6YfuREHL43Dc9gvUqSvxZHE69avL/sdjrm2ST4eM	2026-10-15 02:27:57.117367+05:30
fe4ed670-f43c-4692-94f7-61c1b76b1293	security-test-1792085186@example.com	$argon2id$v=19$m=19456,t=2,p=1$7GVtFNdyhq2B/hiW6XZfpg$8keDzQy7gIo4RjmisISRYY97u8PlnW7nUaQ7TfwvJv8	2026-10-15 22:56:26.618108+05:30
ac73a2ee-9ee3-4177-92b7-ae29891dbb18	ratelimit-test-1792085659@example.com	$argon2id$v=19$m=19456,t=2,p=1$fK6/cBlkTw66B+yvt0RUiQ$Vh7PEXXmB4Fhdefjcmk7qmn/zai3YwdOPgrENQC5LDE	2026-10-15 23:04:20.552397+05:30
f7b27ccc-7e21-4341-8475-12bdbcd809ed	test@example.com	$argon2id$v=19$m=19456,t=2,p=1$s7miLROkkyprkpRHClJRcg$V02gK5lCAoRKZGko6j7TDuz5q+C0jeiZGO5BGAMNt/Q	2026-10-13 00:27:58.388282+05:30
\.


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: yume
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: yume
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: yume
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_token_hash_key UNIQUE (token_hash);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: yume
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: yume
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: idx_sessions_expires_at; Type: INDEX; Schema: public; Owner: yume
--

CREATE INDEX idx_sessions_expires_at ON public.sessions USING btree (expires_at);


--
-- Name: idx_sessions_user_id; Type: INDEX; Schema: public; Owner: yume
--

CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);


--
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: yume
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

\unrestrict z3Egd4luGWpzoel2rXpdFMLIrwaXZIh4SOkGa7eHux1rhNWHUzUbe0bJ3Jz8W6d


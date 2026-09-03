


SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;


COMMENT ON SCHEMA "public" IS 'standard public schema';



CREATE EXTENSION IF NOT EXISTS "pg_stat_statements" WITH SCHEMA "extensions";






CREATE EXTENSION IF NOT EXISTS "pgcrypto" WITH SCHEMA "extensions";






CREATE EXTENSION IF NOT EXISTS "supabase_vault" WITH SCHEMA "vault";






CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA "extensions";






CREATE OR REPLACE FUNCTION "public"."rls_auto_enable"() RETURNS "event_trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'pg_catalog'
    AS $$
DECLARE
  cmd record;
BEGIN
  FOR cmd IN
    SELECT *
    FROM pg_event_trigger_ddl_commands()
    WHERE command_tag IN ('CREATE TABLE', 'CREATE TABLE AS', 'SELECT INTO')
      AND object_type IN ('table','partitioned table')
  LOOP
     IF cmd.schema_name IS NOT NULL AND cmd.schema_name IN ('public') AND cmd.schema_name NOT IN ('pg_catalog','information_schema') AND cmd.schema_name NOT LIKE 'pg_toast%' AND cmd.schema_name NOT LIKE 'pg_temp%' THEN
      BEGIN
        EXECUTE format('alter table if exists %s enable row level security', cmd.object_identity);
        RAISE LOG 'rls_auto_enable: enabled RLS on %', cmd.object_identity;
      EXCEPTION
        WHEN OTHERS THEN
          RAISE LOG 'rls_auto_enable: failed to enable RLS on %', cmd.object_identity;
      END;
     ELSE
        RAISE LOG 'rls_auto_enable: skip % (either system schema or not in enforced list: %.)', cmd.object_identity, cmd.schema_name;
     END IF;
  END LOOP;
END;
$$;


ALTER FUNCTION "public"."rls_auto_enable"() OWNER TO "postgres";

SET default_tablespace = '';

SET default_table_access_method = "heap";


CREATE TABLE IF NOT EXISTS "public"."access_tokens" (
    "token" "text" NOT NULL,
    "created_at" bigint NOT NULL,
    "reserved_email" character varying(255),
    "reserved_at" bigint,
    "redeemed_by" "uuid",
    "redeemed_at" bigint,
    "revoked_at" bigint,
    "expires_at" bigint
);


ALTER TABLE "public"."access_tokens" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."agent_commission_period_status" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "period_key" "text" NOT NULL,
    "upline_role" "text" NOT NULL,
    "status" "text" DEFAULT 'not_yet'::"text" NOT NULL,
    "partial_amount" double precision DEFAULT 0 NOT NULL,
    "partial_note" "text" DEFAULT ''::"text" NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL,
    CONSTRAINT "agent_commission_period_status_role_check" CHECK (("upline_role" = ANY (ARRAY['lead-broker'::"text", 'titling-officer'::"text"]))),
    CONSTRAINT "agent_commission_period_status_status_check" CHECK (("status" = ANY (ARRAY['not_yet'::"text", 'partial'::"text", 'paid'::"text", 'pending'::"text"])))
);


ALTER TABLE "public"."agent_commission_period_status" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."commission_period_status" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "subject_agent_id" "text" NOT NULL,
    "period_start" "date" NOT NULL,
    "period_end" "date" NOT NULL,
    "status" "text" DEFAULT 'not_yet'::"text" NOT NULL,
    "partial_amount" double precision,
    "partial_paid_at" "date",
    "updated_at" bigint DEFAULT 0 NOT NULL,
    "row_key" "text" DEFAULT ''::"text" NOT NULL,
    "paid_at" "date",
    CONSTRAINT "commission_period_status_status_check" CHECK (("status" = ANY (ARRAY['not_yet'::"text", 'partial'::"text", 'pending'::"text", 'paid'::"text"])))
);


ALTER TABLE "public"."commission_period_status" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."commission_rates" (
    "role" character varying(50) NOT NULL,
    "commission_rate" double precision DEFAULT 0 NOT NULL,
    "updated_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL
);


ALTER TABLE "public"."commission_rates" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."commission_release_credits" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "subject_agent_id" "text" NOT NULL,
    "share_kind" "text",
    "amount" double precision NOT NULL,
    "paid_at" "date" NOT NULL,
    "note" "text",
    "created_at" bigint DEFAULT 0 NOT NULL,
    CONSTRAINT "commission_release_credits_amount_check" CHECK (("amount" > (0)::double precision)),
    CONSTRAINT "commission_release_credits_share_kind_check" CHECK ((("share_kind" IS NULL) OR ("share_kind" = ANY (ARRAY['base'::"text", 'pool'::"text", 'promo'::"text"]))))
);


ALTER TABLE "public"."commission_release_credits" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."commission_release_entries" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "subject_agent_id" "text" NOT NULL,
    "period_start" "date" NOT NULL,
    "period_end" "date" NOT NULL,
    "amount" double precision NOT NULL,
    "paid_at" "date" NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "share_kind" "text",
    CONSTRAINT "commission_release_entries_amount_check" CHECK (("amount" > (0)::double precision)),
    CONSTRAINT "commission_release_entries_share_kind_check" CHECK ((("share_kind" IS NULL) OR ("share_kind" = ANY (ARRAY['base'::"text", 'pool'::"text", 'promo'::"text"]))))
);


ALTER TABLE "public"."commission_release_entries" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."commission_row_meta" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "subject_agent_id" "text" NOT NULL,
    "row_key" "text" NOT NULL,
    "period_start" "date" NOT NULL,
    "other_flag" "text" DEFAULT 'none'::"text" NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL,
    "override_amount" double precision,
    CONSTRAINT "commission_row_meta_other_flag_check" CHECK (("other_flag" = ANY (ARRAY['none'::"text", 'half'::"text", 'full'::"text"])))
);


ALTER TABLE "public"."commission_row_meta" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."commission_split_schedule" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "effective_date" "date" NOT NULL,
    "split_months" integer NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL
);


ALTER TABLE "public"."commission_split_schedule" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."company_settings" (
    "id" smallint DEFAULT 1 NOT NULL,
    "company_name" character varying(255) DEFAULT ''::character varying NOT NULL,
    "office_address" "text" DEFAULT ''::"text" NOT NULL,
    "currency" character varying(10) DEFAULT 'PHP'::character varying NOT NULL,
    "timezone" character varying(64) DEFAULT 'Asia/Manila'::character varying NOT NULL,
    "updated_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "agent_commission_split_months" integer DEFAULT 15 NOT NULL,
    CONSTRAINT "company_settings_singleton" CHECK (("id" = 1))
);


ALTER TABLE "public"."company_settings" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."contract_split_history" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "contract_id" "uuid" NOT NULL,
    "split_months" integer NOT NULL,
    "effective_period_start" "date" NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "strategy" "text" DEFAULT 'catch_up'::"text" NOT NULL,
    "rebalance_strategy" "text" DEFAULT 'catch_up'::"text" NOT NULL,
    "late_payment_split_mode" "text" DEFAULT 'keep_due_period_split'::"text" NOT NULL,
    CONSTRAINT "contract_split_history_late_payment_split_mode_check" CHECK (("late_payment_split_mode" = ANY (ARRAY['adopt_new_split'::"text", 'keep_due_period_split'::"text"]))),
    CONSTRAINT "contract_split_history_rebalance_strategy_check" CHECK (("rebalance_strategy" = ANY (ARRAY['even_split'::"text", 'catch_up'::"text"]))),
    CONSTRAINT "contract_split_history_split_months_check" CHECK ((("split_months" >= 1) AND ("split_months" <= 120))),
    CONSTRAINT "contract_split_history_strategy_check" CHECK (("strategy" = ANY (ARRAY['even_split'::"text", 'catch_up'::"text"])))
);


ALTER TABLE "public"."contract_split_history" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."contracts" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "lot_id" "uuid",
    "buyer_name" character varying(255) NOT NULL,
    "buyer_address" "text" DEFAULT ''::"text" NOT NULL,
    "buyer_gmail" character varying(255) DEFAULT ''::character varying NOT NULL,
    "buyer_contact" character varying(50) DEFAULT ''::character varying NOT NULL,
    "lot_block" character varying(20) NOT NULL,
    "lot_lot" character varying(20) NOT NULL,
    "lot_area" double precision DEFAULT 0 NOT NULL,
    "lot_type" character varying(30) DEFAULT 'Inner'::character varying NOT NULL,
    "lot_rate" double precision DEFAULT 0 NOT NULL,
    "contract_price" double precision NOT NULL,
    "payment_plan" character varying(20) DEFAULT 'installment'::character varying NOT NULL,
    "initial_payment" double precision DEFAULT 0 NOT NULL,
    "term_years" integer DEFAULT 0 NOT NULL,
    "monthly_amortization" double precision DEFAULT 0 NOT NULL,
    "due_day" integer DEFAULT 15 NOT NULL,
    "next_due_date" "date" NOT NULL,
    "approval_at" "date",
    "marketing_representative" character varying(255) DEFAULT ''::character varying NOT NULL,
    "agent_code" character varying(100) DEFAULT ''::character varying NOT NULL,
    "selling_agent_id" character varying(255),
    "source_of_buyer" "text"[] DEFAULT '{}'::"text"[] NOT NULL,
    "other_source" character varying(255) DEFAULT ''::character varying NOT NULL,
    "particulars" character varying(255) DEFAULT ''::character varying NOT NULL,
    "created_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "updated_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "agent_id" "uuid",
    "broker_id" "uuid",
    "buyer_last_name" "text" DEFAULT ''::"text" NOT NULL,
    "buyer_first_name" "text" DEFAULT ''::"text" NOT NULL,
    "buyer_middle_name" "text" DEFAULT ''::"text" NOT NULL,
    "term_months" integer DEFAULT 0 NOT NULL,
    "agent_commission_split_months" integer DEFAULT 36 NOT NULL,
    "buyer_user_id" "uuid",
    "is_promo" boolean DEFAULT false NOT NULL,
    "list_price" double precision DEFAULT 0 NOT NULL,
    "amort_start_date" "date",
    "penalty_waived_through_due_date" "date",
    "first_installment_amount" double precision,
    CONSTRAINT "contracts_payment_plan_check" CHECK ((("payment_plan")::"text" = ANY ((ARRAY['installment'::character varying, 'half'::character varying, 'full'::character varying])::"text"[])))
);


ALTER TABLE "public"."contracts" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."employee_position_types" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "label" "text" NOT NULL,
    "sort_order" integer DEFAULT 0 NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL
);


ALTER TABLE "public"."employee_position_types" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."expense_categories" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "name" "text" NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL
);


ALTER TABLE "public"."expense_categories" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."expenses" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "category_id" "uuid" NOT NULL,
    "paid_to" "text" NOT NULL,
    "description" "text",
    "amount" double precision NOT NULL,
    "paid_at" "date" NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    CONSTRAINT "expenses_amount_check" CHECK (("amount" > (0)::double precision))
);


ALTER TABLE "public"."expenses" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."installment_status_overrides" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "contract_id" "uuid" NOT NULL,
    "year" integer NOT NULL,
    "month" integer NOT NULL,
    "status" "text" DEFAULT ''::"text" NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL,
    CONSTRAINT "installment_status_overrides_month_check" CHECK ((("month" >= 0) AND ("month" <= 11))),
    CONSTRAINT "installment_status_overrides_status_check" CHECK (("status" = ANY (ARRAY[''::"text", 'Paid'::"text", 'Half'::"text", 'Hold'::"text"])))
);


ALTER TABLE "public"."installment_status_overrides" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."lots" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "block" character varying(20) NOT NULL,
    "lot" character varying(20) NOT NULL,
    "lot_type" character varying(30) DEFAULT 'Inner'::character varying NOT NULL,
    "area" double precision NOT NULL,
    "rate" double precision NOT NULL,
    "contract_price" double precision NOT NULL,
    "owner_buyer" character varying(255),
    "on_hold" boolean DEFAULT false NOT NULL,
    "status" character varying(20) DEFAULT 'Available'::character varying NOT NULL,
    "created_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "updated_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "reserved_until" timestamp with time zone,
    "reserve_notes" "text" DEFAULT ''::"text" NOT NULL,
    "reserve_agent_id" "text",
    CONSTRAINT "lots_lot_type_check" CHECK ((("lot_type")::"text" = ANY ((ARRAY['Inner'::character varying, 'Commercial'::character varying, 'Corner'::character varying, 'Commercial / Corner'::character varying])::"text"[]))),
    CONSTRAINT "lots_status_check" CHECK ((("status")::"text" = ANY ((ARRAY['Available'::character varying, 'Hold'::character varying, 'Reserved'::character varying, 'Installment'::character varying, 'Sold'::character varying])::"text"[])))
);


ALTER TABLE "public"."lots" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."password_reset_codes" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "email" character varying(255) NOT NULL,
    "code" character varying(6) NOT NULL,
    "expires_at" bigint NOT NULL,
    "created_at" bigint NOT NULL,
    "failed_attempts" integer DEFAULT 0 NOT NULL
);


ALTER TABLE "public"."password_reset_codes" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."payments" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "contract_id" "uuid" NOT NULL,
    "amount" double precision NOT NULL,
    "method" character varying(20) DEFAULT 'cash'::character varying NOT NULL,
    "months_covered" integer DEFAULT 1 NOT NULL,
    "paid_at" "date" NOT NULL,
    "created_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "reference_no" "text" DEFAULT ''::"text" NOT NULL,
    "bank_name" "text" DEFAULT ''::"text" NOT NULL,
    "sender_name" "text" DEFAULT ''::"text" NOT NULL,
    "receiver_name" "text" DEFAULT ''::"text" NOT NULL,
    "mode_label" "text" DEFAULT ''::"text" NOT NULL,
    CONSTRAINT "payments_method_check" CHECK ((("method")::"text" = ANY ((ARRAY['cash'::character varying, 'gcash'::character varying, 'maya'::character varying, 'bank'::character varying, 'others'::character varying])::"text"[])))
);


ALTER TABLE "public"."payments" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."project_rate_categories" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "label" character varying(100) NOT NULL,
    "percent" double precision DEFAULT 0 NOT NULL,
    "is_agent_pool" boolean DEFAULT false NOT NULL,
    "sort_order" integer DEFAULT 0 NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL,
    CONSTRAINT "project_rate_categories_percent_check" CHECK ((("percent" >= (0)::double precision) AND ("percent" <= (100)::double precision)))
);


ALTER TABLE "public"."project_rate_categories" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."project_upline_role_rates" (
    "project_id" "uuid" NOT NULL,
    "upline_role_type_slug" "text" NOT NULL,
    "percent" double precision DEFAULT 0 NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL,
    "has_baseline" boolean,
    "direct_sale_pool_percent" double precision,
    CONSTRAINT "project_upline_role_rates_percent_check" CHECK ((("percent" >= (0)::double precision) AND ("percent" <= (100)::double precision)))
);


ALTER TABLE "public"."project_upline_role_rates" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."projects" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "company_id" smallint DEFAULT 1 NOT NULL,
    "name" character varying(255) NOT NULL,
    "created_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "updated_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "lead_broker_roster_id" "uuid",
    "titling_officer_roster_id" "uuid",
    "agent_commission_split_months" integer DEFAULT 15 NOT NULL,
    "agents_json" "jsonb" DEFAULT '[]'::"jsonb" NOT NULL,
    "subdivision_layout" "jsonb"
);


ALTER TABLE "public"."projects" OWNER TO "postgres";


COMMENT ON COLUMN "public"."projects"."subdivision_layout" IS 'Subdivision editor layout: project boundary, blocks, row/col lines, lot slot positions.';



CREATE TABLE IF NOT EXISTS "public"."roster" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "company_id" smallint DEFAULT 1 NOT NULL,
    "user_id" "uuid" NOT NULL,
    "role" character varying(100) NOT NULL,
    "broker_id" "uuid",
    "code" character varying(100) NOT NULL,
    "prc_license_number" character varying(100),
    "commission_rate" double precision DEFAULT 0 NOT NULL,
    "status" character varying(20) DEFAULT 'Active'::character varying NOT NULL,
    "created_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "updated_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    CONSTRAINT "roster_status_check" CHECK ((("status")::"text" = ANY ((ARRAY['Active'::character varying, 'Inactive'::character varying])::"text"[])))
);


ALTER TABLE "public"."roster" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."salary_employees" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "name" "text" NOT NULL,
    "position" "text",
    "status" "text" DEFAULT 'Active'::"text" NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "user_id" "uuid",
    "rest_days" smallint[] DEFAULT '{}'::smallint[] NOT NULL,
    CONSTRAINT "salary_employees_status_check" CHECK (("status" = ANY (ARRAY['Active'::"text", 'Inactive'::"text"])))
);


ALTER TABLE "public"."salary_employees" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."salary_plans" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "employee_id" "uuid" NOT NULL,
    "kind" "text" NOT NULL,
    "start_date" "date" NOT NULL,
    "end_date" "date",
    "training_fee" double precision,
    "monthly_amount" double precision,
    "schedule_type" "text",
    "created_at" bigint DEFAULT 0 NOT NULL,
    CONSTRAINT "salary_plans_check" CHECK (((("kind" = 'training'::"text") AND ("end_date" IS NOT NULL) AND ("training_fee" IS NOT NULL) AND ("monthly_amount" IS NULL) AND ("schedule_type" IS NULL)) OR (("kind" = 'regular'::"text") AND ("end_date" IS NULL) AND ("monthly_amount" IS NOT NULL) AND ("schedule_type" IS NOT NULL) AND ("training_fee" IS NULL)))),
    CONSTRAINT "salary_plans_kind_check" CHECK (("kind" = ANY (ARRAY['training'::"text", 'regular'::"text"]))),
    CONSTRAINT "salary_plans_monthly_amount_check" CHECK ((("monthly_amount" IS NULL) OR ("monthly_amount" > (0)::double precision))),
    CONSTRAINT "salary_plans_schedule_type_check" CHECK (("schedule_type" = ANY (ARRAY['weekly'::"text", 'semimonthly'::"text", 'monthly'::"text"]))),
    CONSTRAINT "salary_plans_training_fee_check" CHECK ((("training_fee" IS NULL) OR ("training_fee" > (0)::double precision)))
);


ALTER TABLE "public"."salary_plans" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."salary_release_entries" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "project_id" "uuid" NOT NULL,
    "employee_id" "uuid" NOT NULL,
    "period_start" "date" NOT NULL,
    "period_end" "date" NOT NULL,
    "amount" double precision NOT NULL,
    "paid_at" "date" NOT NULL,
    "note" "text",
    "created_at" bigint DEFAULT 0 NOT NULL,
    CONSTRAINT "salary_release_entries_amount_check" CHECK (("amount" > (0)::double precision))
);


ALTER TABLE "public"."salary_release_entries" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."sessions" (
    "id" "uuid" NOT NULL,
    "user_id" "uuid" NOT NULL,
    "created_at" bigint NOT NULL,
    "expires_at" bigint NOT NULL
);


ALTER TABLE "public"."sessions" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."upline_role_types" (
    "slug" "text" NOT NULL,
    "label" "text" NOT NULL,
    "base_commission_percent" double precision DEFAULT 0 NOT NULL,
    "portal_path" "text" NOT NULL,
    "sort_order" integer DEFAULT 0 NOT NULL,
    "created_at" bigint DEFAULT 0 NOT NULL,
    "updated_at" bigint DEFAULT 0 NOT NULL,
    "has_baseline" boolean DEFAULT true NOT NULL,
    "direct_sale_pool_percent" double precision
);


ALTER TABLE "public"."upline_role_types" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."users" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "picture" character varying(255),
    "username" character varying(255) NOT NULL,
    "email" character varying(255) NOT NULL,
    "password_hash" character varying(255) NOT NULL,
    "lastname" character varying(255),
    "firstname" character varying(255),
    "middlename" character varying(255),
    "role" character varying(25) DEFAULT 'User'::character varying NOT NULL,
    "created_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "updated_at" bigint DEFAULT (EXTRACT(epoch FROM "now"()))::bigint NOT NULL,
    "failed_login_attempts" integer DEFAULT 0 NOT NULL,
    "lockout_until" bigint DEFAULT 0 NOT NULL,
    "phone" character varying(32)
);


ALTER TABLE "public"."users" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."verification_codes" (
    "id" "uuid" DEFAULT "gen_random_uuid"() NOT NULL,
    "username" character varying(25) NOT NULL,
    "email" character varying(255) NOT NULL,
    "password_hash" character varying(255) NOT NULL,
    "code" character varying(6) NOT NULL,
    "expires_at" bigint NOT NULL,
    "created_at" bigint NOT NULL,
    "access_token" "text",
    "failed_attempts" integer DEFAULT 0 NOT NULL
);


ALTER TABLE "public"."verification_codes" OWNER TO "postgres";


ALTER TABLE ONLY "public"."access_tokens"
    ADD CONSTRAINT "access_tokens_pkey" PRIMARY KEY ("token");



ALTER TABLE ONLY "public"."agent_commission_period_status"
    ADD CONSTRAINT "agent_commission_period_status_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."agent_commission_period_status"
    ADD CONSTRAINT "agent_commission_period_status_unique" UNIQUE ("project_id", "period_key", "upline_role");



ALTER TABLE ONLY "public"."commission_period_status"
    ADD CONSTRAINT "commission_period_status_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."commission_period_status"
    ADD CONSTRAINT "commission_period_status_unique" UNIQUE ("project_id", "subject_agent_id", "row_key", "period_start");



ALTER TABLE ONLY "public"."commission_rates"
    ADD CONSTRAINT "commission_rates_pkey" PRIMARY KEY ("role");



ALTER TABLE ONLY "public"."commission_release_credits"
    ADD CONSTRAINT "commission_release_credits_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."commission_release_entries"
    ADD CONSTRAINT "commission_release_entries_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."commission_row_meta"
    ADD CONSTRAINT "commission_row_meta_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."commission_row_meta"
    ADD CONSTRAINT "commission_row_meta_unique" UNIQUE ("project_id", "subject_agent_id", "row_key", "period_start");



ALTER TABLE ONLY "public"."commission_split_schedule"
    ADD CONSTRAINT "commission_split_schedule_effective_date_key" UNIQUE ("effective_date");



ALTER TABLE ONLY "public"."commission_split_schedule"
    ADD CONSTRAINT "commission_split_schedule_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."company_settings"
    ADD CONSTRAINT "company_settings_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."contract_split_history"
    ADD CONSTRAINT "contract_split_history_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."contract_split_history"
    ADD CONSTRAINT "contract_split_history_unique" UNIQUE ("contract_id", "effective_period_start");



ALTER TABLE ONLY "public"."contracts"
    ADD CONSTRAINT "contracts_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."employee_position_types"
    ADD CONSTRAINT "employee_position_types_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."expense_categories"
    ADD CONSTRAINT "expense_categories_name_key" UNIQUE ("name");



ALTER TABLE ONLY "public"."expense_categories"
    ADD CONSTRAINT "expense_categories_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."expenses"
    ADD CONSTRAINT "expenses_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."installment_status_overrides"
    ADD CONSTRAINT "installment_status_overrides_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."installment_status_overrides"
    ADD CONSTRAINT "installment_status_overrides_unique" UNIQUE ("contract_id", "year", "month");



ALTER TABLE ONLY "public"."lots"
    ADD CONSTRAINT "lots_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."lots"
    ADD CONSTRAINT "lots_project_id_block_lot_key" UNIQUE ("project_id", "block", "lot");



ALTER TABLE ONLY "public"."password_reset_codes"
    ADD CONSTRAINT "password_reset_codes_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."payments"
    ADD CONSTRAINT "payments_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."project_rate_categories"
    ADD CONSTRAINT "project_rate_categories_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."project_rate_categories"
    ADD CONSTRAINT "project_rate_categories_project_id_label_key" UNIQUE ("project_id", "label");



ALTER TABLE ONLY "public"."project_upline_role_rates"
    ADD CONSTRAINT "project_upline_role_rates_pkey" PRIMARY KEY ("project_id", "upline_role_type_slug");



ALTER TABLE ONLY "public"."projects"
    ADD CONSTRAINT "projects_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."roster"
    ADD CONSTRAINT "roster_code_unique" UNIQUE ("company_id", "code");



ALTER TABLE ONLY "public"."roster"
    ADD CONSTRAINT "roster_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."roster"
    ADD CONSTRAINT "roster_user_id_unique" UNIQUE ("user_id");



ALTER TABLE ONLY "public"."salary_employees"
    ADD CONSTRAINT "salary_employees_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."salary_plans"
    ADD CONSTRAINT "salary_plans_employee_id_start_date_key" UNIQUE ("employee_id", "start_date");



ALTER TABLE ONLY "public"."salary_plans"
    ADD CONSTRAINT "salary_plans_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."salary_release_entries"
    ADD CONSTRAINT "salary_release_entries_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."sessions"
    ADD CONSTRAINT "sessions_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."upline_role_types"
    ADD CONSTRAINT "upline_role_types_pkey" PRIMARY KEY ("slug");



ALTER TABLE ONLY "public"."users"
    ADD CONSTRAINT "users_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."users"
    ADD CONSTRAINT "users_username_key" UNIQUE ("username");



ALTER TABLE ONLY "public"."verification_codes"
    ADD CONSTRAINT "verification_codes_pkey" PRIMARY KEY ("id");



CREATE INDEX "agent_commission_period_status_project_idx" ON "public"."agent_commission_period_status" USING "btree" ("project_id");



CREATE INDEX "commission_period_status_project_period_idx" ON "public"."commission_period_status" USING "btree" ("project_id", "period_start", "period_end");



CREATE UNIQUE INDEX "commission_rates_role_unique" ON "public"."commission_rates" USING "btree" ("role");



CREATE INDEX "commission_release_credits_project_subject_idx" ON "public"."commission_release_credits" USING "btree" ("project_id", "subject_agent_id");



CREATE INDEX "commission_release_entries_project_period_idx" ON "public"."commission_release_entries" USING "btree" ("project_id", "subject_agent_id", "period_start");



CREATE INDEX "commission_row_meta_project_idx" ON "public"."commission_row_meta" USING "btree" ("project_id", "subject_agent_id", "period_start");



CREATE INDEX "contract_split_history_contract_idx" ON "public"."contract_split_history" USING "btree" ("contract_id", "effective_period_start");



CREATE UNIQUE INDEX "employee_position_types_label_unique" ON "public"."employee_position_types" USING "btree" ("label");



CREATE INDEX "expenses_category_idx" ON "public"."expenses" USING "btree" ("category_id");



CREATE INDEX "idx_access_tokens_expires_at" ON "public"."access_tokens" USING "btree" ("expires_at");



CREATE INDEX "idx_access_tokens_redeemed_by" ON "public"."access_tokens" USING "btree" ("redeemed_by");



CREATE INDEX "idx_access_tokens_reserved_email" ON "public"."access_tokens" USING "btree" ("reserved_email");



CREATE INDEX "idx_contracts_agent_id" ON "public"."contracts" USING "btree" ("agent_id");



CREATE INDEX "idx_contracts_broker_id" ON "public"."contracts" USING "btree" ("broker_id");



CREATE INDEX "idx_contracts_buyer_user_id" ON "public"."contracts" USING "btree" ("buyer_user_id");



CREATE INDEX "idx_contracts_lot_id" ON "public"."contracts" USING "btree" ("lot_id");



CREATE INDEX "idx_contracts_project_id" ON "public"."contracts" USING "btree" ("project_id");



CREATE INDEX "idx_lots_project_id" ON "public"."lots" USING "btree" ("project_id");



CREATE UNIQUE INDEX "idx_password_reset_codes_email" ON "public"."password_reset_codes" USING "btree" ("email");



CREATE INDEX "idx_payments_contract_id" ON "public"."payments" USING "btree" ("contract_id");



CREATE INDEX "idx_projects_company_id" ON "public"."projects" USING "btree" ("company_id");



CREATE INDEX "idx_roster_broker_id" ON "public"."roster" USING "btree" ("broker_id");



CREATE INDEX "idx_roster_company_id" ON "public"."roster" USING "btree" ("company_id");



CREATE INDEX "idx_sessions_expires_at" ON "public"."sessions" USING "btree" ("expires_at");



CREATE INDEX "idx_sessions_user_id" ON "public"."sessions" USING "btree" ("user_id");



CREATE INDEX "idx_verification_codes_access_token" ON "public"."verification_codes" USING "btree" ("access_token");



CREATE INDEX "idx_verification_codes_email" ON "public"."verification_codes" USING "btree" ("email");



CREATE INDEX "installment_status_overrides_project_idx" ON "public"."installment_status_overrides" USING "btree" ("project_id", "year");



CREATE UNIQUE INDEX "project_rate_categories_one_agent_pool" ON "public"."project_rate_categories" USING "btree" ("project_id") WHERE "is_agent_pool";



CREATE UNIQUE INDEX "salary_employees_user_id_unique" ON "public"."salary_employees" USING "btree" ("user_id") WHERE ("user_id" IS NOT NULL);



CREATE INDEX "salary_plans_employee_idx" ON "public"."salary_plans" USING "btree" ("employee_id", "start_date");



CREATE INDEX "salary_release_entries_project_idx" ON "public"."salary_release_entries" USING "btree" ("project_id", "employee_id", "period_start");



CREATE UNIQUE INDEX "upline_role_types_label_unique" ON "public"."upline_role_types" USING "btree" ("label");



CREATE UNIQUE INDEX "upline_role_types_portal_path_unique" ON "public"."upline_role_types" USING "btree" ("portal_path");



ALTER TABLE ONLY "public"."agent_commission_period_status"
    ADD CONSTRAINT "agent_commission_period_status_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."commission_period_status"
    ADD CONSTRAINT "commission_period_status_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."commission_release_credits"
    ADD CONSTRAINT "commission_release_credits_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."commission_release_entries"
    ADD CONSTRAINT "commission_release_entries_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."commission_row_meta"
    ADD CONSTRAINT "commission_row_meta_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."contract_split_history"
    ADD CONSTRAINT "contract_split_history_contract_id_fkey" FOREIGN KEY ("contract_id") REFERENCES "public"."contracts"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."contracts"
    ADD CONSTRAINT "contracts_agent_id_fkey" FOREIGN KEY ("agent_id") REFERENCES "public"."roster"("id") ON DELETE SET NULL;



ALTER TABLE ONLY "public"."contracts"
    ADD CONSTRAINT "contracts_broker_id_fkey" FOREIGN KEY ("broker_id") REFERENCES "public"."roster"("id") ON DELETE SET NULL;



ALTER TABLE ONLY "public"."contracts"
    ADD CONSTRAINT "contracts_buyer_user_id_fkey" FOREIGN KEY ("buyer_user_id") REFERENCES "public"."users"("id");



ALTER TABLE ONLY "public"."contracts"
    ADD CONSTRAINT "contracts_lot_id_fkey" FOREIGN KEY ("lot_id") REFERENCES "public"."lots"("id") ON DELETE SET NULL;



ALTER TABLE ONLY "public"."contracts"
    ADD CONSTRAINT "contracts_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."expenses"
    ADD CONSTRAINT "expenses_category_id_fkey" FOREIGN KEY ("category_id") REFERENCES "public"."expense_categories"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."installment_status_overrides"
    ADD CONSTRAINT "installment_status_overrides_contract_id_fkey" FOREIGN KEY ("contract_id") REFERENCES "public"."contracts"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."installment_status_overrides"
    ADD CONSTRAINT "installment_status_overrides_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."lots"
    ADD CONSTRAINT "lots_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."payments"
    ADD CONSTRAINT "payments_contract_id_fkey" FOREIGN KEY ("contract_id") REFERENCES "public"."contracts"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."project_rate_categories"
    ADD CONSTRAINT "project_rate_categories_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."project_upline_role_rates"
    ADD CONSTRAINT "project_upline_role_rates_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."project_upline_role_rates"
    ADD CONSTRAINT "project_upline_role_rates_upline_role_type_slug_fkey" FOREIGN KEY ("upline_role_type_slug") REFERENCES "public"."upline_role_types"("slug") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."projects"
    ADD CONSTRAINT "projects_company_id_fkey" FOREIGN KEY ("company_id") REFERENCES "public"."company_settings"("id");



ALTER TABLE ONLY "public"."projects"
    ADD CONSTRAINT "projects_lead_broker_roster_id_fkey" FOREIGN KEY ("lead_broker_roster_id") REFERENCES "public"."roster"("id");



ALTER TABLE ONLY "public"."projects"
    ADD CONSTRAINT "projects_titling_officer_roster_id_fkey" FOREIGN KEY ("titling_officer_roster_id") REFERENCES "public"."roster"("id");



ALTER TABLE ONLY "public"."roster"
    ADD CONSTRAINT "roster_broker_id_fkey" FOREIGN KEY ("broker_id") REFERENCES "public"."roster"("id") ON DELETE SET NULL;



ALTER TABLE ONLY "public"."roster"
    ADD CONSTRAINT "roster_company_id_fkey" FOREIGN KEY ("company_id") REFERENCES "public"."company_settings"("id");



ALTER TABLE ONLY "public"."roster"
    ADD CONSTRAINT "roster_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."salary_employees"
    ADD CONSTRAINT "salary_employees_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE SET NULL;



ALTER TABLE ONLY "public"."salary_plans"
    ADD CONSTRAINT "salary_plans_employee_id_fkey" FOREIGN KEY ("employee_id") REFERENCES "public"."salary_employees"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."salary_release_entries"
    ADD CONSTRAINT "salary_release_entries_employee_id_fkey" FOREIGN KEY ("employee_id") REFERENCES "public"."salary_employees"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."salary_release_entries"
    ADD CONSTRAINT "salary_release_entries_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects"("id") ON DELETE CASCADE;



ALTER TABLE "public"."access_tokens" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."agent_commission_period_status" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."commission_period_status" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."commission_rates" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."commission_release_credits" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."commission_release_entries" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."commission_row_meta" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."commission_split_schedule" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."company_settings" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."contract_split_history" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."contracts" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."employee_position_types" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."expense_categories" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."expenses" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."installment_status_overrides" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."lots" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."password_reset_codes" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."payments" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."project_rate_categories" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."project_upline_role_rates" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."projects" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."roster" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."salary_employees" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."salary_plans" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."salary_release_entries" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."sessions" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."upline_role_types" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."users" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."verification_codes" ENABLE ROW LEVEL SECURITY;




ALTER PUBLICATION "supabase_realtime" OWNER TO "postgres";


GRANT USAGE ON SCHEMA "public" TO "postgres";
GRANT USAGE ON SCHEMA "public" TO "anon";
GRANT USAGE ON SCHEMA "public" TO "authenticated";
GRANT USAGE ON SCHEMA "public" TO "service_role";






















































































































































GRANT ALL ON FUNCTION "public"."rls_auto_enable"() TO "anon";
GRANT ALL ON FUNCTION "public"."rls_auto_enable"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."rls_auto_enable"() TO "service_role";


















GRANT ALL ON TABLE "public"."access_tokens" TO "anon";
GRANT ALL ON TABLE "public"."access_tokens" TO "authenticated";
GRANT ALL ON TABLE "public"."access_tokens" TO "service_role";



GRANT ALL ON TABLE "public"."agent_commission_period_status" TO "anon";
GRANT ALL ON TABLE "public"."agent_commission_period_status" TO "authenticated";
GRANT ALL ON TABLE "public"."agent_commission_period_status" TO "service_role";



GRANT ALL ON TABLE "public"."commission_period_status" TO "anon";
GRANT ALL ON TABLE "public"."commission_period_status" TO "authenticated";
GRANT ALL ON TABLE "public"."commission_period_status" TO "service_role";



GRANT ALL ON TABLE "public"."commission_rates" TO "anon";
GRANT ALL ON TABLE "public"."commission_rates" TO "authenticated";
GRANT ALL ON TABLE "public"."commission_rates" TO "service_role";



GRANT ALL ON TABLE "public"."commission_release_credits" TO "anon";
GRANT ALL ON TABLE "public"."commission_release_credits" TO "authenticated";
GRANT ALL ON TABLE "public"."commission_release_credits" TO "service_role";



GRANT ALL ON TABLE "public"."commission_release_entries" TO "anon";
GRANT ALL ON TABLE "public"."commission_release_entries" TO "authenticated";
GRANT ALL ON TABLE "public"."commission_release_entries" TO "service_role";



GRANT ALL ON TABLE "public"."commission_row_meta" TO "anon";
GRANT ALL ON TABLE "public"."commission_row_meta" TO "authenticated";
GRANT ALL ON TABLE "public"."commission_row_meta" TO "service_role";



GRANT ALL ON TABLE "public"."commission_split_schedule" TO "anon";
GRANT ALL ON TABLE "public"."commission_split_schedule" TO "authenticated";
GRANT ALL ON TABLE "public"."commission_split_schedule" TO "service_role";



GRANT ALL ON TABLE "public"."company_settings" TO "anon";
GRANT ALL ON TABLE "public"."company_settings" TO "authenticated";
GRANT ALL ON TABLE "public"."company_settings" TO "service_role";



GRANT ALL ON TABLE "public"."contract_split_history" TO "anon";
GRANT ALL ON TABLE "public"."contract_split_history" TO "authenticated";
GRANT ALL ON TABLE "public"."contract_split_history" TO "service_role";



GRANT ALL ON TABLE "public"."contracts" TO "anon";
GRANT ALL ON TABLE "public"."contracts" TO "authenticated";
GRANT ALL ON TABLE "public"."contracts" TO "service_role";



GRANT ALL ON TABLE "public"."employee_position_types" TO "anon";
GRANT ALL ON TABLE "public"."employee_position_types" TO "authenticated";
GRANT ALL ON TABLE "public"."employee_position_types" TO "service_role";



GRANT ALL ON TABLE "public"."expense_categories" TO "anon";
GRANT ALL ON TABLE "public"."expense_categories" TO "authenticated";
GRANT ALL ON TABLE "public"."expense_categories" TO "service_role";



GRANT ALL ON TABLE "public"."expenses" TO "anon";
GRANT ALL ON TABLE "public"."expenses" TO "authenticated";
GRANT ALL ON TABLE "public"."expenses" TO "service_role";



GRANT ALL ON TABLE "public"."installment_status_overrides" TO "anon";
GRANT ALL ON TABLE "public"."installment_status_overrides" TO "authenticated";
GRANT ALL ON TABLE "public"."installment_status_overrides" TO "service_role";



GRANT ALL ON TABLE "public"."lots" TO "anon";
GRANT ALL ON TABLE "public"."lots" TO "authenticated";
GRANT ALL ON TABLE "public"."lots" TO "service_role";



GRANT ALL ON TABLE "public"."password_reset_codes" TO "anon";
GRANT ALL ON TABLE "public"."password_reset_codes" TO "authenticated";
GRANT ALL ON TABLE "public"."password_reset_codes" TO "service_role";



GRANT ALL ON TABLE "public"."payments" TO "anon";
GRANT ALL ON TABLE "public"."payments" TO "authenticated";
GRANT ALL ON TABLE "public"."payments" TO "service_role";



GRANT ALL ON TABLE "public"."project_rate_categories" TO "anon";
GRANT ALL ON TABLE "public"."project_rate_categories" TO "authenticated";
GRANT ALL ON TABLE "public"."project_rate_categories" TO "service_role";



GRANT ALL ON TABLE "public"."project_upline_role_rates" TO "anon";
GRANT ALL ON TABLE "public"."project_upline_role_rates" TO "authenticated";
GRANT ALL ON TABLE "public"."project_upline_role_rates" TO "service_role";



GRANT ALL ON TABLE "public"."projects" TO "anon";
GRANT ALL ON TABLE "public"."projects" TO "authenticated";
GRANT ALL ON TABLE "public"."projects" TO "service_role";



GRANT ALL ON TABLE "public"."roster" TO "anon";
GRANT ALL ON TABLE "public"."roster" TO "authenticated";
GRANT ALL ON TABLE "public"."roster" TO "service_role";



GRANT ALL ON TABLE "public"."salary_employees" TO "anon";
GRANT ALL ON TABLE "public"."salary_employees" TO "authenticated";
GRANT ALL ON TABLE "public"."salary_employees" TO "service_role";



GRANT ALL ON TABLE "public"."salary_plans" TO "anon";
GRANT ALL ON TABLE "public"."salary_plans" TO "authenticated";
GRANT ALL ON TABLE "public"."salary_plans" TO "service_role";



GRANT ALL ON TABLE "public"."salary_release_entries" TO "anon";
GRANT ALL ON TABLE "public"."salary_release_entries" TO "authenticated";
GRANT ALL ON TABLE "public"."salary_release_entries" TO "service_role";



GRANT ALL ON TABLE "public"."sessions" TO "anon";
GRANT ALL ON TABLE "public"."sessions" TO "authenticated";
GRANT ALL ON TABLE "public"."sessions" TO "service_role";



GRANT ALL ON TABLE "public"."upline_role_types" TO "anon";
GRANT ALL ON TABLE "public"."upline_role_types" TO "authenticated";
GRANT ALL ON TABLE "public"."upline_role_types" TO "service_role";



GRANT ALL ON TABLE "public"."users" TO "anon";
GRANT ALL ON TABLE "public"."users" TO "authenticated";
GRANT ALL ON TABLE "public"."users" TO "service_role";



GRANT ALL ON TABLE "public"."verification_codes" TO "anon";
GRANT ALL ON TABLE "public"."verification_codes" TO "authenticated";
GRANT ALL ON TABLE "public"."verification_codes" TO "service_role";









ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "postgres";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "anon";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "authenticated";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "service_role";






ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "postgres";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "anon";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "authenticated";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "service_role";






ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON TABLES TO "postgres";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON TABLES TO "anon";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON TABLES TO "authenticated";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON TABLES TO "service_role";




































use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction, Pool, Row};

use crate::query_client::Query;
use crate::{log::LogStructure, query_client};
use crate::{
    query_client::LikeEqual,
    query_client::EqualBetween,
    query_client::EqualOp,
    query_client::LikeOp,
    query_client::BetweenOp,
};

pub async fn insert_logs(tx: &mut Transaction<'_, Postgres>, loglist: Vec<LogStructure>) {
    // TODO: Replace this with something less terrible
    let insertable = convert_logs_to_sql(loglist);
    let q = format!(r#"
            insert into public.logs ("level", "message", "resource_id", "timestamp", "trace_id", "span_id", "commit", "parent_resource_id") values {}
        "#, insertable);
    sqlx::query(&q).execute(&mut **tx).await.unwrap();
}

fn convert_logs_to_sql(loglist: Vec<LogStructure>) -> String {
    // TODO: Replace this with something less terrible
    let mut sql = String::new();
    for log in loglist {
        sql += &format!(
            "('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}'),",
            log.level, log.message, log.resource_id,
            log.timestamp, log.trace_id, log.span_id, log.commit, log.metadata.parent_resource_id);
    }
    sql.pop();
    sql
}


pub fn create_sql_conditions(query: query_client::Query) -> String {
    let mut conditions: Vec<String> = Vec::new();

    if let Some(level) = query.level {
        conditions.push(match level {
            LikeEqual::Equal(EqualOp{equals}) => format!("level = '{}'", equals),
            LikeEqual::Like(LikeOp{like}) => format!("level like '{}'", like)
        });
    }

    if let Some(message) = query.message {
        conditions.push(match message {
            LikeEqual::Equal(EqualOp{equals}) => format!("message = '{}'", equals),
            LikeEqual::Like(LikeOp{like}) => format!("message like '{}'", like)
        });
    }

    if let Some(resource_id) = query.resource_id {
        conditions.push(match resource_id {
            LikeEqual::Equal(EqualOp{equals}) => format!("resource_id = '{}'", equals),
            LikeEqual::Like(LikeOp{like}) => format!("resource_id like '{}'", like)
        });
    }

    if let Some(trace_id) = query.trace_id {
        conditions.push(match trace_id {
            LikeEqual::Equal(EqualOp{equals}) => format!("trace_id = '{}'", equals),
            LikeEqual::Like(LikeOp{like}) => format!("trace_id like '{}'", like)
        });
    }

    if let Some(span_id) = query.span_id {
        conditions.push(match span_id {
            LikeEqual::Equal(EqualOp{equals}) => format!("span_id = '{}'", equals),
            LikeEqual::Like(LikeOp{like}) => format!("span_id like '{}'", like)
        });
    }

    if let Some(commit) = query.commit {
        conditions.push(match commit {
            LikeEqual::Equal(EqualOp{equals}) => format!("commit = '{}'", equals),
            LikeEqual::Like(LikeOp{like}) => format!("commit like '{}'", like)
        });
    }
    
    if let Some(metadata) = query.metadata {
        if let Some(parent_resource_id) = metadata.parent_resource_id {
            conditions.push(match parent_resource_id {
                LikeEqual::Equal(EqualOp{equals}) => format!("parent_resource_id = '{}'", equals),
                LikeEqual::Like(LikeOp{like}) => format!("parent_resource_id like '{}'", like)
            });
        }
    }

    if let Some(timestamp) = query.timestamp {
        conditions.push(match timestamp {
            EqualBetween::Equal(EqualOp{equals}) => format!("timestamp = '{}'", equals),
            EqualBetween::Between(BetweenOp{from, to}) => format!("timestamp between '{}' AND '{}'", from, to)
        });
    }
    conditions.join(" AND ")

}



pub async fn query_logs(pool: &Pool<Postgres>, query: Query) -> Vec<LogStructure> {
    let top = query.pagination.top;
    let offset = query.pagination.offset;
    let mut condition_string = create_sql_conditions(query);

    if condition_string.len() == 0 {
        condition_string = "true=true".to_string();
    }

    let query = format!("select * from logs where {} limit {} offset {}",
                        condition_string,
                        top,
                        offset
                    );
    println!("Query: {}", query);

    let result = sqlx::query(&query).fetch_all(pool)
        .await.unwrap();
    let ret = result.iter().map(|r| LogStructure {
        level: r.get::<String, _>("level"),
        message: r.get::<String, _>("message"),
        trace_id: r.get::<String, _>("trace_id"),
        span_id: r.get::<String, _>("span_id"),
        resource_id: r.get::<String, _>("resource_id"),
        commit: r.get::<String, _>("commit"),
        timestamp: r.get::<DateTime<Utc>, _>("timestamp").into(),
        metadata: crate::log::LogMetadata {
            parent_resource_id: r.get::<String, _>("parent_resource_id")
        }
    })
    .collect::<Vec<LogStructure>>();
    ret
}
